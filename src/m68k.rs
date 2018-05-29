//Roland Ballinger - roland2@pdx.edu
//M68K Processor Emulator
//CS 410P - Rust Programming

////////////////////////////////m68k.rs////////////////////////////////
//  This file contains the struct 'm68k', the members and methods of //
//  which contain the core functionality of the emulator. Also found //
//  in this file is st struct Mem, which is a generic memory struct  //
//  representing the M68K's ability to write to 16mb of ram. Not all //
//  systems using this CPU have that amount, but for the purpose of  //
//  testing I thought it would be best to allow the emulator to use  //
//  the full gamut of what any program written for the processor     //
//  would e able to use. The method 'run' is the main loop for the   //
//  emulator, and handles the parsing of opcodes and calling of the  //
//  various mnemonics. The methods of class m68k aside from run and  //
//  a couple of other utility methods are all named after the M68K   //
//  mnemonics they replace, and can be found  under those names.     //
///////////////////////////////////////////////////////////////////////

use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

pub struct M68k {
    a: [u32; 8],
    d: [u32; 8],
    pc: u32,
    x: bool,
    n: bool,
    z: bool,
    v: bool,
    c: bool,
    s: bool,
    op: u16,
    prog: Vec<u8>,
    memory: Mem,
}

impl M68k {
    pub fn init() -> M68k {
        M68k {
            a: [0 as u32; 8],
            d: [0 as u32; 8],
            pc: 0 as u32,
            x: false,
            n: false,
            z: false,
            v: false,
            c: false,
            s: false,
            op: 0 as u16,
            prog: Vec::new(),
            memory: Mem::new(),
        }
    }

    pub fn load(&mut self, mut file: File) -> Result<(), ()> {
        file.read_to_end(&mut self.prog);
        self.pc = 0;
        Ok(())
    }

    fn next_op(&mut self) -> u16 {
        let temp: u16 =
            ((self.prog[self.pc as usize] as u16) << 8) + self.prog[self.pc as usize + 1] as u16;
        self.pc += 2;
        temp
    }

    //This method is extremely important, and represents the core functional
    //loop. Each loop, the variable op gets the next opcode from the program
    //file. The first 4 digits of op are read to determine the type of op being
    //performed. The encoding of the rest of the op is dependent on the op
    //itself, so each opcode has its own parsing rules. The corresponding
    //match blocks are commented with the mnemonic being decoded.
    pub fn run(&mut self) -> bool {
        self.op = self.next_op();
        println!("{}", self.pc);
        debug_print(self);
        let temp = self.op;
        if temp == 0 {
            return true;
        }
        match (temp.rotate_left(4) & 0b1111) {
            0b0000 => {
                //beginning of IMMEDIATE OPS block
                match (temp.rotate_left(8) & 0b1111) {
                    0b0000 => {
                        self.ori();
                    } // ori
                    0b0010 => {
                        self.andi();
                    } //andi
                    0b0100 => {
                        self.subi();
                    } //subi
                    0b0110 => {
                        self.addi();
                    } //addi
                    0b1010 => {
                        self.eori();
                    } //eori
                    0b1100 => {
                        self.cmpi();
                    } //cmpi
                    _ => {}
                }
            }
            0b0001 => {
                self.mov();
            } //move b
            0b0010 => {
                self.mov();
            } //move l
            0b0011 => {
                self.mov();
            } //move w
            0b0100 => match temp {
                0x4afc => {
                    self.illegal();
                    return false;
                }
                0x4e71 => {} //nop
                0x4e72 => {
                    self.stop();
                    return false;
                }
                0x4e73 => {
                    self.rte();
                }
                0x4e75 => {
                    self.rts();
                }
                0x4e76 => {
                    self.trapv();
                }
                0x4e77 => {
                    self.rtr();
                }
                //REQ BITS: 9
                temp if (temp & 0b111111111000) == 0b111001010000 => {
                    self.link();
                }
                temp if (temp & 0b111111111000) == 0b111001011000 => {
                    self.unlk();
                }
                temp if (temp & 0b111111111000) == 0b100001000000 => {
                    self.swap();
                }
                //REQ BITS: 8
                temp if (temp & 0b111111110000) == 0b111001000000 => {
                    self.trap();
                }
                //REQ BITS: 6
                temp if (temp & 0b111111000000) == 0b111011000000 => {
                    self.jmp();
                }
                temp if (temp & 0b111111000000) == 0b111010000000 => {
                    self.jsr();
                }
                temp if (temp & 0b111111000000) == 0b111001000000 => {
                    self.tas();
                }
                temp if (temp & 0b111111000000) == 0b100001000000 => {
                    self.pea();
                }
                temp if (temp & 0b111000111000) == 0b100000000000 => {
                    self.ext();
                }
                //REQ BITS: 4
                temp if (temp & 0b111100000000) == 0b101000000000 => {
                    self.tst();
                }
                temp if (temp & 0b111100000000) == 0b011000000000 => {
                    self.not();
                }
                temp if (temp & 0b111100000000) == 0b010000000000 => {
                    self.neg();
                }
                temp if (temp & 0b111100000000) == 0b001000000000 => {
                    self.clr();
                }
                _ => self.lea(),
            },
            0b0101 => match temp {
                temp if (temp & 0b11000000) == 0b11000000 => {
                    self.scc();
                }
                temp if (temp & 0b100000000) == 0b100000000 => {
                    self.subq();
                }
                _ => {
                    self.addq();
                }
            },
            0b0110 => {
                self.bcc();
            }
            0b0111 => {
                self.d[((temp >> 9) & 0b111) as usize] = (temp & 0b11111111).into();
            }
            0b1000 => {
                self.div();
            }
            0b1001 => {
                self.suba();
            }
            0b1100 => {
                if (temp & 0b11111111) == 0b11111100 {
                    self.mul();
                } else {
                    self.exg();
                }
            }
            0b1101 => {
                self.adda();
            }
            0b1110 => {
                if (temp & 0b11000) == 0b11000 {
                    self.rot();
                } else {
                    self.ls();
                }
            }
            _ => {
                println!("uncaptured address or data! previous ops:");
                println!(
                    "{:b}{:b}",
                    self.prog[(self.pc - 4) as usize],
                    self.prog[(self.pc - 3) as usize]
                );
                println!(
                    "{:b}{:b}",
                    self.prog[(self.pc - 2) as usize],
                    self.prog[(self.pc - 1) as usize]
                );
            }
        }
        return true;
    }

    fn ori(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        if arg == 0x007c {
            //ori with SR
            self.x = (((temp >> 4) & 0b1) != 0) | self.x;
            self.n = (((temp >> 3) & 0b1) != 0) | self.n;
            self.z = (((temp >> 2) & 0b1) != 0) | self.z;
            self.v = (((temp >> 1) & 0b1) != 0) | self.v;
            self.c = ((temp & 0b1) != 0) | self.c;
            return;
        }
        self.v = false;
        self.c = false;
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                //ori with d register
                self.d[reg] = by_byte((self.d[reg] | temp as u32), self.d[reg], (arg >> 6) & 0b11);
                match (arg >> 6) & 0b11 {
                    0b00 => {
                        //flags for byte
                        let check = self.d[reg];
                        self.z = (check & 0xff == 0);
                        self.n = (check & 0x80 != 0);
                    }
                    0b01 => {
                        //setting flags for word
                        let check = self.d[reg];
                        self.z = (check & 0xffff == 0);
                        self.n = (check & 0x8000 != 0);
                    }
                    0b10 => {
                        //setting flags for longword
                        let check = self.d[reg];
                        self.z = (check == 0);
                        self.n = (check & 0x80000000 != 0);
                    }
                    _ => {}
                }
                self.v = false;
                self.c = false;
            }
            0b111 => {
                //ORI with memory
                let temp2 = self.next_op();
                self.d[reg] = by_byte(
                    (temp as u32 | self.memory.readl(temp2 as usize)),
                    self.d[reg],
                    (arg >> 6) & 0b11,
                );
            }
            _ => {}
        }
    }

    fn andi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        if arg == 0x027c {
            self.x = (((temp >> 4) & 0b1) != 0) && self.x;
            self.n = (((temp >> 3) & 0b1) != 0) && self.n;
            self.z = (((temp >> 2) & 0b1) != 0) && self.z;
            self.v = (((temp >> 1) & 0b1) != 0) && self.v;
            self.c = ((temp & 0b1) != 0) && self.c;
            return;
        }
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                self.d[reg] = by_byte((self.d[reg] & temp as u32), self.d[reg], (arg >> 6) & 0b11);
            }
            0b111 => {
                //andi with memory
                let temp2 = self.next_op();
                self.d[reg] = by_byte(
                    (temp as u32 & self.memory.readl(temp2 as usize)),
                    self.d[reg],
                    (arg >> 6) & 0b11,
                );
            }
            _ => {}
        }
    }
    fn subi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                let res =
                    (self.d[reg] as i32 - (by_byte(temp as u32, 0, (arg >> 6) & 0b11) as i32));
                self.n = res < 0;
                self.z = res == 0;
                self.v = res > self.d[reg] as i32;
                self.c = self.v;
                self.d[reg] = by_byte(res as u32, self.d[reg], (arg >> 6) & 0b11);
            }
            0b111 => {
                //subi with memory
                let temp2 = self.next_op();
                let mut res = by_byte(self.memory.readl(temp2 as usize), 0, arg >> 6);
                res = (self.d[reg] as i32 - res as i32) as u32;
                self.n = res < 0;
                self.z = res == 0;
                self.v = res > self.d[reg];
                self.c = self.v;
                self.d[reg] = by_byte(res, self.d[reg], (arg >> 6) & 0b11);
            }
            _ => {}
        }
    }

    fn addi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                let res = (self.d[reg] as i32 + temp as i32);
                match arg >> 6 & 0b11 {
                    0b00 => {
                        let check = (res & 0xff) as i8;
                        self.v = res > 0xff;
                        self.x = self.c;
                        self.c = (res & 0x100) != 0;
                        self.z = check == 0;
                        self.n = check < 0;
                    }
                    0b01 => {
                        let check = (res & 0xffff) as i16;
                        self.c = (res & 0x10000) != 0;
                        self.x = self.c;
                        self.v = res > 0xffff;
                        self.z = check == 0;
                        self.n = check < 0;
                    }
                    0b10 => {
                        let check = (res < temp as i32) | (res < self.d[reg] as i32);
                        self.c = check;
                        self.v = check;
                        self.x = check;
                        self.z = res == 0;
                        self.n = res < 0;
                    }
                    _ => {}
                }
                self.d[reg] = by_byte(res as u32, self.d[reg], (arg >> 6) & 0b11);
            }
            0b111 => {
                let temp2 = self.next_op();
                println!(
                    "Should be doing addi with memory at {:x} and
                the immediate number {:x}",
                    temp2, temp
                );
            }
            _ => {}
        }
    }

    fn eori(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        if arg == 0x0A7c {
            self.x = (((temp >> 4) & 0b1) != 0) ^ self.x;
            self.n = (((temp >> 3) & 0b1) != 0) ^ self.n;
            self.z = (((temp >> 2) & 0b1) != 0) ^ self.z;
            self.v = (((temp >> 1) & 0b1) != 0) ^ self.v;
            self.c = ((temp & 0b1) != 0) ^ self.c;
            return;
        }
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                self.d[reg] = by_byte((self.d[reg] ^ temp as u32), self.d[reg], (arg >> 6) & 0b11);
            }
            0b111 => {
                let temp2 = self.next_op();
                let temp3 = self.memory.readl(temp2 as usize) ^ (temp as u32);
                self.memory.mem_write(temp2 as usize, temp3, 4);
            }
            _ => {}
        }
    }

    fn cmpi(&mut self) {
        let arg = self.op;
        let temp = self.next_op(); //this is the immediate
        let temp2 = self.next_op(); //this is the place in memory to read from, or 0 if we're working on a register
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                let res = (self.d[reg] as i32 - temp as i32);
                match arg >> 6 & 0b11 {
                    0b00 => {
                        let check = (res & 0xff) as i8;
                        self.v = res as u32 > 0xff;
                        self.c = (res & 0x100) != 0;
                        self.z = (check == 0);
                        self.n = (check < 0);
                    }
                    0b01 => {
                        let check = (res & 0xffff) as i16;
                        self.v = res as u32 > 0xffff;
                        self.c = (res & 0x10000) != 0;
                        self.z = (check == 0);
                        self.n = (check < 0);
                    }
                    0b10 => {
                        let check = (self.d[reg] < res as u32) | (temp as u32 > res as u32);
                        self.x = check;
                        self.v = check;
                        self.c = check;
                        self.z = res == 0;
                        self.n = res < 0;
                    }
                    _ => {}
                }
            }
            0b111 => {
                println!(
                    "Should be doing cmpi with memory at {:x} and
                the immediate number {:x}",
                    temp2, temp
                );
            }
            _ => {}
        }
    }

    fn mov(&mut self) {
        let arg = self.op;
        let mut source: u32 = 0;
        match ((arg >> 3) & 0b111) {
            //finding source
            0 => {
                //source is a d register
                source += self.d[(arg & 0b111) as usize];
            }
            0b001 => {
                //source is an a reg
            }
            0b010 => {
                //source is an address in an a reg
            }
            0b011 => {
                //source is an address in an a reg with post offset
            }
            0b100 => {
                //source is a reg with pre offset
            }
            0b101 => {
                //source is an address with displacement
            }
            0b110 => {
                //source is an address with index
            }
            0b111 => {
                //source is immediate
                match ((arg >> 12) & 0b11) {
                    0b10 => {
                        //longword immediate
                        source += ((self.next_op() as u32) << 16) + self.next_op() as u32;
                    }
                    //word and byte immediates are both coded as 16 bits, so they're
                    //treated as the same case here
                    _ => {
                        source += self.next_op() as u32;
                    }
                }
            }
            _ => {
                println!("uncaught move source");
            }
        }

        match ((arg >> 6) & 0b111) {
            //finding dest
            0 => {
                //dest is a d register
                self.d[((arg >> 9) & 0b111) as usize] = source;
            }
            0b111 => {
                //dest is memory
                let mut mode = 0;
                match (arg >> 12) {
                    0b01 => mode = 1,
                    0b11 => mode = 2,
                    0b10 => mode = 4,
                    _ => {}
                }
                let temp = self.next_op() as usize;
                self.memory.mem_write(temp, source, mode);
            }
            0b001 => {
                //dest is an a reg
                self.a[((arg >> 9) & 0b111) as usize] = source;
            }
            _ => {
                println!("uncaught move dest");
            }
        }
    }

    fn rte(&mut self) {
        let SR = self.memory.readl(self.a[7] as usize);
        self.a[7] -= 4;
        let PC = self.memory.readl(self.a[7] as usize);
        self.a[7] -= 4;
    }

    fn rtr(&mut self) {
        println!("not sure how this is different from RTE");
        self.rte();
    }
    fn illegal(&mut self) {
        println!("this is a type of trap: the PC and SR are pushed to stack, and can be retrieved with RTE");
        self.a[7] -= 4;
        self.memory.mem_write(self.a[7] as usize, self.pc, 4);
        self.a[7] -= 4;
        let mut temp = 0;
        if self.c {
            temp += 0b1;
        }
        if self.v {
            temp += 0b10;
        }
        if self.z {
            temp += 0b100;
        }
        if self.n {
            temp += 0b1000;
        }
        if self.x {
            temp += 0b10000;
        }
        if self.s {
            temp += 0x8000;
        }
        self.memory.mem_write(self.a[7] as usize, temp, 4);
    }

    fn stop(&mut self) {
        if self.s == true {
            self.x = false;
            self.n = false;
            self.z = false;
            self.v = false;
            self.c = false;
            self.op = 0x007c;
            self.ori();
        } else {
            self.trap();
        }
    }

    fn rts(&mut self) {
        self.pc = self.memory.readl(self.a[7] as usize);
        self.a[7] += 4;
    }

    fn unlk(&mut self) {
        let arg = self.op & 0b111;
        self.a[7] = self.a[arg as usize];
        let temp = self.memory.readw(self.a[arg as usize] as usize) as u32;
        self.a[arg as usize] = temp;
    }

    fn link(&mut self) {
        let arg = self.op & 0b111;
        self.a[7] -= 4;
        self.memory
            .mem_write(self.a[7] as usize, self.a[arg as usize], 4);
        self.a[arg as usize] = self.a[7];
        self.a[7] += self.next_op() as u32;
    }

    fn swap(&mut self) {
        let arg = self.op & 0b111;
        let temp = self.d[arg as usize].rotate_left(16);
        self.d[arg as usize] = temp;
        self.n = (temp & 0x8000) != 0;
        self.c = false;
        self.z = temp != 0;
    }

    fn trap(&mut self) {
        self.s = true;
        let arg = self.op & 0xf;
        println!("Call to trap # {}", arg);
    }

    fn trapv(&mut self) {
        if self.v {
            self.trap();
        }
    }

    fn jmp(&mut self) {
        let arg = self.op & 0b111111;
        match arg & 0b111000 {
            0 => {
                //jump to address in an A reg
                self.pc = self.a[(arg & 0b111) as usize];
            }
            0b111000 => {
                //jump to imm address
                match arg & 0b111 {
                    0b001 => {
                        //jump to long word
                        self.pc = ((self.next_op() as u32) << 16) + self.next_op() as u32;
                    }
                    0b000 => {
                        //jump to word
                        self.pc = (self.next_op() as u32);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn jsr(&mut self) {
        self.a[7] -= 4;
        self.memory.mem_write(self.a[7] as usize, self.pc, 4);
        self.jmp();
    }

    fn tas(&mut self) {
        let temp = self.next_op() as usize;
        if (self.memory.readb(temp) == 0) {
            self.memory.mem_write(temp, 80, 1);
        }
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
        let arg = self.op;
        let mut offset = 0 as u16;
        match arg & 0b11111111 {
            0 => {
                offset = self.next_op();
            } // 16 bit offset
            _ => offset = arg & 0b11111111, //8 bit offset
        }
        offset = offset << 1;
        match (arg >> 8) & 0b1111 {
            0b0000 => {
                self.pc += offset as u32;
            } //bra
            0b0001 => {
                self.a[7] -= 4;
                self.memory.mem_write(self.a[7] as usize, self.pc, 4);
                self.pc += offset as u32;
            }
            0b0010 => if (!self.c && !self.z) {
                self.pc += offset as u32;
            }, //bhi
            0b0011 => if (self.c | self.z) {
                self.pc += offset as u32;
            }, //bls
            0b0100 => if !self.c {
                self.pc += offset as u32;
            }, //bcc
            0b0101 => if self.c {
                self.pc += offset as u32;
            }, //bcs
            0b0110 => if !self.z {
                self.pc += offset as u32;
            }, //bne
            0b0111 => if self.z {
                self.pc += offset as u32;
            }, //beq
            0b1000 => if !self.v {
                self.pc += offset as u32;
            }, //bvc
            0b1001 => if self.v {
                self.pc += offset as u32;
            }, //bvs
            0b1010 => if !self.n {
                self.pc += offset as u32;
            }, //bpl
            0b1011 => if self.n {
                self.pc += offset as u32;
            }, //bmi
            0b1100 => {
                if (self.n && self.v) | (!self.n && !self.v) {
                    self.pc += offset as u32;
                }
            } //bge
            0b1101 => {
                if (self.n && !self.v) | (!self.n && self.v) {
                    self.pc += offset as u32;
                }
            } //blt
            0b1110 => {
                if (self.n && self.v && !self.z) | (!self.n && !self.v && !self.z) {
                    self.pc += offset as u32;
                }
            } //bgt
            0b1111 => {
                if self.z | (self.n && !self.v) | (!self.n && self.v) {
                    self.pc += offset as u32;
                }
            } //ble
            _ => {
                println!("If you're here you blew it! Opcode {}", arg);
            }
        }
    }

    fn div(&mut self) {}
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

fn by_byte(from: u32, to: u32, mode: u16) -> u32 {
    match mode {
        0b10 => return from,
        0b01 => {
            let temp = from & 0b00000000000000001111111111111111;
            let temp2 = to & 0b11111111111111110000000000000000;
            return temp + temp2;
        }
        0b00 => {
            let temp = from & 0b00000000000000000000000011111111;
            let temp2 = to & 0b11111111111111111111111100000000;
            return temp + temp2;
        }
        _ => 0,
    }
}

pub fn debug_print(test: &M68k) {
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
    println!("xnzvc: {}{}{}{}{}", test.x, test.n, test.z, test.v, test.c);
}

pub struct Mem {
    m: Vec<u8>,
}

impl Mem {
    pub fn new() -> Mem {
        Mem {
            m: vec![0; 0xffffff],
        }
    }

    pub fn mem_write(&mut self, addr: usize, data: u32, mode: u32) {
        match mode {
            4 => {
                self.m[addr + 3] = (data >> 24) as u8;
                self.m[addr + 2] = (data >> 16) as u8;
                self.m[addr + 1] = (data >> 8) as u8;
                self.m[addr] = data as u8;
            }
            2 => {
                self.m[addr + 1] = (data >> 8) as u8;
                self.m[addr] = data as u8;
            }
            1 => {
                self.m[addr] = data as u8;
            }
            _ => {}
        }
    }

    pub fn readb(&mut self, addr: usize) -> u8 {
        //The CPU leaves data in an array of u8s, and whatever needs to see
        //those values can request them here. This is a stopgap solution.
        self.m[addr]
    }

    pub fn readw(&mut self, addr: usize) -> u16 {
        ((self.m[addr + 1] as u16) << 8) + (self.m[addr] as u16)
    }

    pub fn readl(&mut self, addr: usize) -> u32 {
        ((self.readw(addr + 2) as u32) << 16) + (self.readw(addr) as u32)
    }
}
