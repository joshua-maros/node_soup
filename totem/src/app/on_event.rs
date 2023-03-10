use itertools::Itertools;
use maplit::hashmap;
use renderer::{
    winit::{
        ControlFlow, ElementState, Event, KeyboardInput, MouseButton, PhysicalPosition,
        VirtualKeyCode, WindowEvent,
    },
    Position,
};

use super::App;
use crate::{
    engine::{TypedBlob, Node, NodeId, NodeOperation, ParameterId, ToolId},
    widgets::BoundingBoxKind,
};

impl App {
    pub(super) fn on_event(&mut self, event: Event<()>) {
        match event {
            Event::RedrawRequested(_window_id) => self.render(),
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                if let WindowEvent::Resized(new_size) = event {
                    self.render_engine.resize_target(new_size.into())
                } else {
                    self.on_window_event(event)
                }
            }
            Event::MainEventsCleared => self.window.request_redraw(),
            _ => (),
        }
    }

    fn physical_pos_to_render_pos(&self, pos: PhysicalPosition<f64>) -> Position {
        Position {
            x: pos.x as f32,
            y: self.render_engine.target_size().height - pos.y as f32,
        }
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_input(input),
            WindowEvent::CursorMoved { position, .. } => {
                self.on_mouse_move(self.physical_pos_to_render_pos(position))
            }
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(button, state),
            _ => (),
        }
    }

    fn on_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => self.on_key_down(code),
                ElementState::Released => self.on_key_up(code),
            }
        }
    }

    fn on_key_down(&mut self, code: VirtualKeyCode) {
        match code {
            VirtualKeyCode::Escape => self.control_flow = ControlFlow::Exit,
            _ => (),
        }
    }

    fn on_key_up(&mut self, code: VirtualKeyCode) {
        match code {
            _ => (),
        }
    }

    fn on_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.on_mouse_down(button),
            ElementState::Released => self.on_mouse_up(button),
        }
    }

    fn on_mouse_down(&mut self, button: MouseButton) {
        if button == MouseButton::Left {
            self.dragging = self.hovering.clone();
            if let &Some(BoundingBoxKind::InvokeTool(tool_id)) = &self.dragging {
                let tool = &self.computation_engine.get_tool(tool_id);
                let target_prototype = tool.target_prototype;
                self.collapse_to_literal = None;
                if let Ok(targets) =
                    self.match_prototype_to_node(target_prototype, self.active_node())
                {
                    self.tool_targets = targets;
                } else {
                    let should_collapse = if let NodeOperation::Literal(..) =
                        &self.computation_engine[self.active_node()].operation
                    {
                        true
                    } else {
                        false
                    };
                    let old_active = self.active_node();
                    let new_active = self.insert_prototype(target_prototype, self.active_node());
                    if should_collapse {
                        self.collapse_to_literal = Some((old_active, new_active));
                    }
                    *self.selected_node_path.last_mut().unwrap() = new_active;
                    self.tool_targets = self
                        .match_prototype_to_node(target_prototype, new_active)
                        .unwrap();
                }
            }
        }
    }

    fn match_prototype_to_node(
        &self,
        prototype: NodeId,
        node: NodeId,
    ) -> Result<Vec<(ParameterId, NodeId)>, ()> {
        let prototype = &self.computation_engine[prototype];
        if let &NodeOperation::Parameter(param_id) = &prototype.operation {
            if let Ok(name) = &self.computation_engine[prototype.arguments[0]]
                .as_literal()
                .view()
                .as_string()
            {
                if name.starts_with("SPECIAL TOOL TARGET ") {
                    if let NodeOperation::Literal(..) = &self.computation_engine[node].operation {
                        return Ok(vec![(param_id, node)]);
                    }
                } else if name.starts_with("SPECIAL TOOL WILDCARD") {
                    return Ok(vec![]);
                }
            }
        }
        let node = &self.computation_engine[node];
        if prototype.operation == node.operation {
            assert_eq!(prototype.arguments.len(), node.arguments.len());
            assert_eq!(prototype.input.is_some(), node.input.is_some());
            let mut result = vec![];
            if let Some(prototype_input) = prototype.input {
                result.append(
                    &mut self.match_prototype_to_node(prototype_input, node.input.unwrap())?,
                );
            }
            for index in 0..prototype.arguments.len() {
                result.append(
                    &mut self.match_prototype_to_node(
                        prototype.arguments[index],
                        node.arguments[index],
                    )?,
                );
            }
            return Ok(result);
        }
        Err(())
    }

    fn insert_prototype(&mut self, prototype: NodeId, after: NodeId) -> NodeId {
        let (prototype_instance, instance_bottom) = self.instantiate_prototype(prototype, after);
        self.replace_references(after, prototype_instance);
        self.computation_engine[instance_bottom.unwrap()].input = Some(after);
        self.computation_engine.mark_dirty(instance_bottom.unwrap());
        prototype_instance
    }

    fn replace_references(&mut self, to: NodeId, with: NodeId) {
        for node in self.computation_engine.nodes_mut() {
            for arg in node.arguments.iter_mut().chain(node.input.iter_mut()) {
                if *arg == to {
                    *arg = with;
                }
            }
        }
        if self.computation_engine.root_node() == to {
            self.computation_engine.set_root(with);
        }
        if let Some(position) = self.selected_node_path.iter().position(|node| *node == to) {
            self.selected_node_path.resize(position, to);
            self.selected_node_path.push(with);
            assert!(!self.selected_node_path.contains(&to));
        }
    }

    fn instantiate_prototype(
        &mut self,
        prototype_id: NodeId,
        root_input: NodeId,
    ) -> (NodeId, Option<NodeId>) {
        let prototype = &self.computation_engine[prototype_id];
        if let NodeOperation::Parameter(..) = &prototype.operation {
            let name = self.computation_engine[prototype.arguments[0]]
                .as_literal()
                .view();
            let name = name.as_string().unwrap();
            if name.starts_with("SPECIAL TOOL TARGET ") {
                return self.instantiate_prototype(prototype.input.unwrap(), root_input);
            } else if name.starts_with("SPECIAL TOOL WILDCARD INPUT") {
                return (prototype_id, None);
            }
        }
        let operation = prototype.operation.clone();
        let this_input = prototype.input;
        let arguments = prototype.arguments.clone();
        let (input, bottommost_node) = if let Some(input) = this_input {
            let (input, bottommost_node) = self.instantiate_prototype(input, root_input);
            (Some(input), bottommost_node)
        } else {
            (None, None)
        };
        let node = Node {
            operation,
            input,
            arguments: arguments
                .into_iter()
                .map(|arg| self.instantiate_prototype(arg, root_input).0)
                .collect_vec(),
        };
        let instance = self.computation_engine.push_node(node);
        (instance, Some(bottommost_node.unwrap_or(instance)))
    }

    fn on_mouse_up(&mut self, button: MouseButton) {
        if button == MouseButton::Left {
            if let Some(BoundingBoxKind::SelectNode(index, node)) = self.dragging {
                assert!(index <= self.selected_node_path.len());
                self.selected_node_path.resize(index, node);
                self.selected_node_path.push(node);
                assert_eq!(self.selected_node_path.last(), Some(&node));
            } else if let Some(BoundingBoxKind::InvokeTool(..)) = self.dragging {
                if let Some((old_literal, output)) = self.collapse_to_literal {
                    let mut io = self.computation_engine.default_io_blob(output);
                    self.computation_engine.execute(output, &mut io);
                    let value = io.view().index(&TypedBlob::from(format!("OUTPUT"))).to_owned();
                    self.computation_engine
                        .write_constant_data(old_literal, value.clone());
                    *self.computation_engine[old_literal].as_literal_mut() = value;
                    self.computation_engine.mark_dirty(output);
                    self.replace_references(output, old_literal);
                }
            }
            self.dragging = None;
        }
    }

    fn on_mouse_move(&mut self, new_pos: Position) {
        let dx = new_pos.x - self.previous_mouse_pos.x;
        let dy = new_pos.y - self.previous_mouse_pos.y;
        let d = (dx, dy);
        self.previous_mouse_pos = new_pos;
        if let Some(target) = &self.dragging {
            match target {
                &BoundingBoxKind::InvokeTool(tool_id) => {
                    self.drag_tool(tool_id, d);
                }
                _ => (),
            }
        } else {
            self.update_hovering();
        }
    }

    fn update_hovering(&mut self) {
        self.hovering = None;
        let mut candidate = &self.root_bbox;
        'look_into_candidate: while let BoundingBoxKind::Parent(children) = &candidate.kind {
            for child in children {
                if child.contains(self.previous_mouse_pos) {
                    candidate = child;
                    continue 'look_into_candidate;
                }
            }
            return;
        }
        self.hovering = Some(candidate.kind.clone());
    }

    fn drag_tool(&mut self, tool: ToolId, d: (f32, f32)) {
        let target_id = self.tool_targets[0].1;
        let target_value = self.computation_engine[target_id].as_literal().clone();
        let encoded_delta = TypedBlob::fixed_heterogeneous_map(vec![
            (format!("X").into(), d.0.into()),
            (format!("Y").into(), d.1.into()),
        ]);
        let tool = self.computation_engine.get_tool(tool);
        let mut io = TypedBlob::fixed_heterogeneous_map(vec![
            (format!("OUTPUT").into(), 0.0.into()),
            (format!("INPUT Mouse Offset").into(), encoded_delta.clone()),
            (
                format!("INPUT SPECIAL TOOL TARGET Factor").into(),
                target_value,
            ),
        ]);
        self.computation_engine
            .execute(tool.mouse_drag_handler, &mut io);
        let new_data = io.view().index(&format!("OUTPUT").into()).to_owned();
        self.computation_engine
            .write_constant_data(target_id, new_data.clone());
        let target = &mut self.computation_engine[target_id];
        *target.as_literal_mut() = new_data;
    }
}
