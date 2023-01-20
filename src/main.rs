mod init;

pub fn main() {
    pollster::block_on(init::run());
}
