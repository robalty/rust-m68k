//Roland Ballinger
//M68K Processor Emulator
//CS 410P - Rust Programming


extern crate bytes;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io;
use std::env;



fn main() {
    let mut params = env::args();
    params.next();
    let file: File = File::open(&(params.next()).unwrap()).unwrap();
    let mut myCPU = M68k::init();
    myCPU.load(file);
    let mut i: u64 = 0;
    while (myCPU.run() == true) {

    }
}


struct M68k {
    a: [u32; 8],
    d: [u32; 8],
    pc: u32,
    x: bool,
    n: bool,
    z: bool,
    v: bool,
    c: bool,
    op: u16,
    prog: Vec<u8>,
}


impl M68k {
    fn init() -> M68k {
        M68k{
            a: [0 as u32; 8],
            d: [0 as u32; 8],
            pc: 0 as u32,
            x: false,
            n: false,
            z: false,
            v: false,
            c: false,
            op: 0 as u16,
            prog: Vec::with_capacity(2_u32.pow(22) as usize),
        } 
    }

    fn load(&mut self, mut file: File) -> Result<(), ()>{
        file.read_to_end(&mut self.prog);
        self.op = 0 as u16;
        Ok(())
    }

    //This method is extremely important, and represents the core functional
    //loop. Each loop, the variable op gets the next opcode from the program
    //file. The first 4 digits of op are read to determine the type of op being
    //performed. The encoding of the rest of the op is dependent on the op 
    //itself, so each opcode has its own parsing rules. The corresponding
    //match blocks are commented with the mnemonic being decoded.
    fn run(&mut self) -> bool{
        if self.pc >= self.prog.len() as u32 {
            return false;
        }
        self.op = ((self.prog[self.pc as usize] as u16) << 8) 
            + self.prog[self.pc as usize +1] as u16;
        self.pc += 2;
        let temp = self.op;
        if temp == 0 { return true; }
        match (temp.rotate_left(4) & 0b1111) {
            0b0000 => { //beginning of IMMEDIATE OPS block
                match (temp.rotate_left(8) & 0b1111) {
                    0b0000 => { self.ori();  }, // ori
                    0b0010 => { self.andi(); }, //andi
                    0b0100 => { self.subi(); }, //subi
                    0b0110 => { self.addi();}, //addi
                    0b1010 => { self.eori();  }, //eori
                    0b1100 => { self.cmpi(); }, //cmpi
                    _ => { },
                }
            },
            0b0001 => { self.mov(); },//move b
            0b0010 => { self.mov(); },//move l
            0b0011 => { self.mov(); },//move w
            0b0100 => match temp {
                0x4afc => {
                    self.illegal();
                    return false;
                },
                0x4e71 => {}, //nop
                0x4e72 => {
                    self.stop();
                    return false;
                },
                0x4e73 => {
                    self.rte();
                },
                0x4e75 => {
                    self.rts();
                },
                0x4e76 => {
                    self.trapv();
                },
                0x4e77 => {
                    self.rtr();
                }
                //REQ BITS: 9
                temp if (temp & 0b111111111000) == 0b111001010000 => {
                    self.link();
                },
                temp if (temp & 0b111111111000) == 0b111001011000 => {
                    self.unlk();
                },
                temp if (temp & 0b111111111000) == 0b100001000000 => {
                    self.swap();
                },
                //REQ BITS: 8
                temp if (temp & 0b111111110000) == 0b111001000000 => {
                    self.trap();
                },
                //REQ BITS: 6
                temp if (temp & 0b111111000000) == 0b111011000000 => {
                    self.jmp();
                },
                temp if (temp & 0b111111000000) == 0b111010000000 => {
                    self.jsr();
                },
                temp if (temp & 0b111111000000) == 0b111001000000 => {
                    self.tasb();
                },
                temp if (temp & 0b111111000000) == 0b100001000000 => {
                    self.pea();
                },
                temp if (temp & 0b111000111000) == 0b100000000000 => {
                    self.ext();
                },
                //REQ BITS: 4
                temp if (temp & 0b111100000000) == 0b101000000000 => {
                    self.tst();
                },
                temp if (temp & 0b111100000000) == 0b011000000000 => {
                    self.not();
                },
                temp if (temp & 0b111100000000) == 0b010000000000 => {
                    self.neg();
                },
                temp if (temp & 0b111100000000) == 0b001000000000 => {
                    self.clr();
                },
                _ => {self.lea()},
            }
            0b0101 => { match temp {

                temp if (temp & 0b11000000) == 0b11000000 => {
                    self.scc();
                }
                temp if (temp & 0b100000000) == 0b100000000 => {
                    self.subq();
                },
                _ => {self.addq();},
            }
            },
            0b0110 => { self.bcc(); },
            0b0111 => { self.d[((temp >> 9) & 0b111) as usize] = 
                (temp & 0b11111111).into();       
            },
            0b1000 => { self.div(); 
            },
            0b1001 => { self.suba(); },
            0b1100 => {
                if (temp & 0b11111111) == 0b11111100 {
                    self.mul();
                }else { self.exg();}
            },
            0b1101 => { self.adda(); },
            0b1110 => {
                if (temp & 0b11000) == 0b11000 {
                self.rot();
                }else{ self.ls();}
            },
            _ => { println!("unknown op!"); },
        }
        return true;
        
    }

    fn ori(&mut self) {
        println!("ori");
    }
    fn andi(&mut self) {
        println!("andi");
    }
    fn subi(&mut self) {
        println!("subi");
    }
    fn addi(&mut self) {
        println!("addi");
    }
    fn eori(&mut self) {
        println!("eori");
    }
    fn cmpi(&mut self) {
        println!("cmpi");
    }
    fn mov(&mut self) {
        println!("mov");
    }
    fn rte(&mut self) {
        println!("rte");
    }
    fn rtr(&mut self) {
        println!("rtr");
    }
    fn illegal(&mut self) {
        println!("illegal");
    }
    fn stop(&mut self) {
        println!("stop");
    }
    fn rts(&mut self) {
        println!("rts");
    }
    fn unlk(&mut self) {
        println!("unlk");
    }
    fn link(&mut self) {
        println!("link");
    } 
    fn swap(&mut self) {
        println!("swap");
    }
    fn trap(&mut self) {
        println!("trap");
    } 
    fn trapv(&mut self) {
        println!("trapv");
    }   
    fn jmp(&mut self) {
        println!("jump");
    }
    fn jsr(&mut self) {
        println!("jsr");
    }
    fn tasb(&mut self) {
        println!("tasb");
    }
    fn pea(&mut self) {
        println!("pea");
    }
    fn ext(&mut self) {
        println!("ext");
    }
    fn tst(&mut self) {
        println!("tst");
    }
    fn not(&mut self) {
        println!("not");
    }
    fn neg(&mut self) {
        println!("neg");
    }
    fn clr(&mut self) {
        println!("clr");
    }
    fn lea(&mut self) {
        println!("lea");
    }
    fn scc(&mut self) {
        println!("scc");
    }
    fn subq(&mut self) {
        println!("subq");
    }
    fn addq(&mut self) {
        println!("addq");
    }
    fn bcc(&mut self) {
        println!("bcc");
    }
    fn div(&mut self) {
        println!("div");
    }
    fn suba(&mut self) {
        println!("suba");
    }
    fn mul(&mut self) {
        println!("mul");
    }
    fn exg(&mut self) {
        println!("exg");
    }
    fn adda(&mut self) {
        println!("adda");
    }
    fn rot(&mut self) {
        println!("rot");
    }
    fn ls(&mut self) {
        println!("ls");
    }
}



fn by_byte(from: u32, to: u32, mode: u32) -> u32{
    match mode {
        0 => return from,
        1 => {  let temp = from & 0b00000000000000001111111111111111; 
            let temp2 = to  & 0b11111111111111110000000000000000;
            return temp + temp2;
        },
        2 => {  let temp = from & 0b00000000000000000000000011111111; 
            let temp2 =  to & 0b11111111111111111111111100000000;
            return temp + temp2;
        },
        _ => 0,
    }
}


#[test]
fn test(){
}


fn debug_print(test: &M68k){
    let mut i = 0;
    for x in (*test).a.iter() {
        println!("A{}: {:X}", i, x);
        i += 1;
    }
    i = 0;
    for x in (*test).d.iter() {
        println!("D{}: {:X}", i, x);
        i += 1;
    }
}
