mod init;
pub mod constants;

pub fn main() {
    pollster::block_on(init::run());
}
