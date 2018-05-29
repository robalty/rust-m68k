use std::env;
use std::fs::File;
use std::io;
mod m68k;

fn main() {
    let mut params = env::args();
    params.next();
    let file: File = File::open(&(params.next()).unwrap()).unwrap();
    let mut myCPU = m68k::M68k::init();
    myCPU.load(file);
    while (myCPU.run() == true) {}
    m68k::debug_print(&myCPU);
}
