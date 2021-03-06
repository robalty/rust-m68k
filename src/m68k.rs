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
    pc: u32, //program counter
    sr: u16, //status register - bits are:
    //0: carry, 1: overflow, 2: zero, 3: negative, 4: extend, 5-14: ???, 15: trace enabled
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
            sr: 0 as u16,
            op: 0 as u16,
            prog: Vec::new(),
            memory: Mem::new(),
        }
    }

    pub fn load(&mut self, mut file: File) -> Result<(), ()> {
        file.read_to_end(&mut self.prog);
        self.a[7] = 0xffffff;
        self.pc = 0;
        Ok(())
    }

    fn next_op(&mut self) -> u16 {
        let temp: u16 =
            ((self.prog[self.pc as usize] as u16) << 8) + 
                self.prog[self.pc as usize + 1] as u16;
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
        println!("{}", self.pc);
        debug_print(&self);
        self.op = self.next_op();
        match self.op {
            0 => {return true;}
            0b0100101011111100 => {self.illegal(); return false;},
            0b0100111001110000 => {self.reset(); return true;},
            0b0100111001110001 => {return true;}//this is a nop
            0b0100111001110010 => {self.stop(); return false;},
            0b0100111001110011 => {self.rte(); return true;},
            0b0100111001110101 => {self.rts(); return true;},
            0b0100111001110110 => {self.trapv(); return false;},
            0b0100111001110111 => {self.rtr(); return true;},
            _ => {}
        }
        match (self.op >> 12) & 0xf {

            0 => {
                //immediate operation
                match (self.op >> 8) & 0xf {
                    0 => self.ori(),
                    0b0010 => self.andi(),
                    0b0100 => self.subi(),
                    0b0110 => self.addi(),
                    0b1010 => self.eori(),
                    0b1100 => self.cmpi(),
                    0b1000 => {
                        match (self.op >> 6) & 0b111 {//dest for these is the Z bit of the SR
                            0 => self.btst(),
                            1 => self.bchg(),
                            2 => self.bclr(),
                            3 => self.bset(),
                            _ => {}
                        }
                    }
                    _ => self.movep(), //this is the only other immediate op
                }
            }
            0b0001 | 0b0010 | 0b0011 => self.mov(),
            0b0100 => {
                //this block contains LOTS of misc operations
                match self.op{
                    op if (op & 0b111111111000) == 0b111001010000 => self.link(),
                    op if (op & 0b111111111000) == 0b111001011000 => self.unlk(),
                    op if (op & 0b111111110000) == 0b111001000000 => self.trap(),
                    op if (op & 0b111111000000) == 0b101011000000 => self.tas(),
                    op if (op & 0b111111000000) == 0b111010000000 => self.jsr(),
                    op if (op & 0b111111000000) == 0b111011000000 => self.jmp(),
                    op if (op & 0b101110000000) == 0b100010000000 => self.movem(),
                    op if (op & 0b111100000000) == 0b101000000000 => self.tst(),
                    op if (op & 0b111000000) == 0b111000000 => self.lea(),
                    op if (op & 0b111000000) == 0b110000000 => self.chk(),
                    _ => {}, //still a couple to be implemented but i don't know what they do
                }
            }
            0b0101 => {
                if (self.op >> 6)&0b11 == 0b11 {
                    if (self.op >> 3) &0b111 == 0b001 {
                        self.dbcc();
                    }
                    else {
                        self.scc();
                    }
                }
                else{
                    if (self.op & 0x100) == 0 {
                        self.addq();
                    }
                    else{
                        self.subq();
                    }
                }
            }
            0b0110 => self.bcc(),
            0b0111 => {
                //moveq
                let reg = ((self.op >>9) & 0b111) as usize;
                self.d[reg] = (self.op & 0xFF) as u32;
            }
            0b1000 => {
                // div, decimal subtraction, bitwise or
                if(self.op >> 6) &0b11 == 0b11 {
                    self.div();
                }
                else if (self.op >> 4) & 0b1111 == 0b1000{
                    self.sbcd();
                }
                else{ self.or(); }
            }
            0b1001 => {
                self.sub();
            }
            0b1011 => {
                if (self.op & 0b100000) == 0b100000 {
                    if(self.op &0b111) != 001 {
                        self.eor();
                    }
                }
                self.cmp();
            }
            0b1100 => {
                let temp = self.op >> 4;
                //multiplication, extended decimal addition, and, exg
                match temp {
                    temp if ((temp >> 2) & 0b111 == 0b011) => self.mul(),
                    temp if ((temp >> 2) & 0b111 == 0b111) => self.mul(),
                    temp if (temp &0b11111 == 0b10000) => self.abcd(),
                    temp if (temp &0b10011 == 0b10000) => self.exg(),
                    _ => self.and(),
                }
            }
            0b1101 => {
                //addition
                self.add();

            }
            0b1110 => {
                //shifts and rotations

            }
            _ => println!("is this an opcode? {:#018b}", self.op),		
        } 
        return true;
    }

    fn ori(&mut self) {
        let arg = self.op;
        let mode: u32 = 1 << ((arg >> 6) & 0b11) as u32;
        let mut temp = 0;
        if mode == 4 {
            temp = self.next_l();
        }
        else{
            temp = self.next_op() as u32;
        }
        if arg == 0x007c {
            self.sr = self.sr | temp as u16;
            return;
        }
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                //ori with d register
                self.d[reg] = by_byte((self.d[reg] | temp), self.d[reg], mode);
            }
            0b111 => {
                //ORI with memory
                let temp2 = self.next_op();
                self.d[reg] = by_byte((temp | self.memory.read_l(temp2 as usize)), 
                                      self.d[reg], mode);
            }
            _ => {}
        }
    }

    fn andi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        if arg == 0x027c {

            return;
        }
        let mode: u32 = 1 << ((arg >> 6) & 0b11) as u32;
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                self.d[reg] = by_byte((self.d[reg] & temp as u32), self.d[reg], mode);
            }
            0b111 => {
                //andi with memory
                let temp2 = self.next_op();
                self.d[reg] = by_byte((temp as u32 & self.memory.read_l(temp2 as usize)), self.d[reg], mode);
            }
            _ => {}
        }
    }

    fn subi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        let reg: usize = (arg & 0b111) as usize;
        let  mode = 1 << ((arg >> 6) & 0b11);
        match (arg >> 3) & 0b111 {
            0 => {
                let res =
                    (self.d[reg] as i32 - (by_byte(temp as u32, 0, mode) as i32));
                self.d[reg] = by_byte(res as u32, self.d[reg], mode);
            }
            0b111 => {
                //subi with memory
                let temp2 = self.next_op();
                let mut res = by_byte(self.memory.read_l(temp2 as usize), 0, mode);
                res = (self.d[reg] as i32 - res as i32) as u32;
                self.d[reg] = by_byte(res, self.d[reg], mode);
            }
            _ => {}
        }
    }

    fn btst(&mut self){
        let bitnum = self.next_op() as u32;
        let reg = (self.op & 0b111) as usize;
        match (self.op >> 3) & 0b111 {//finding source
            0 => {//data register
                let mask = 2_u32.pow(bitnum % 32);
                if(self.d[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            1 => {//A register
                let mask = 2_u32.pow(bitnum % 32);
                if(self.a[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            2 => {//address from A reg
                let mask = 2_u8.pow(bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            3 => {//A(n) with increment
                let mask = 2_u8.pow(bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.a[reg] += 1;
            }
            4 => {//A(n) with decrement
                let mask = 2_u8.pow(bitnum % 7);
                self.a[reg] -= 1;
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            _ => println!("invalid addressing mode for BTSTZ")

        }
    }

    fn bchg(&mut self){
        let bitnum = self.next_op() as u32;
        let reg = (self.op & 0b111) as usize;
        match (self.op >> 3) & 0b111 {//finding source
            0 => {//data register
                let mask = 1 << (bitnum % 32);
                if(self.d[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.d[reg] = self.d[reg] ^ mask;
            }
            1 => {//A register
                let mask = 1 << (bitnum % 32);
                if(self.a[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.a[reg] = self.a[reg] ^ mask;
            }
            2 => {//address from A reg
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp ^ mask) as u32, 1);
            }
            3 => {//A(n) with increment
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp ^ mask)as u32, 1);
                self.a[reg] += 1;
            }
            4 => {//A(n) with decrement
                let mask = 1 << (bitnum % 7);
                self.a[reg] -= 1;
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp ^ mask)as u32, 1);
            }
            _ => {println!("invalid addressing mode for BCHGZ");}

        }
    }

    fn bclr(&mut self){
        let bitnum = self.next_op() as u32;
        let reg = (self.op & 0b111) as usize;
        match (self.op >> 3) & 0b111 {//finding source
            0 => {//data register
                let mask = 2_u32.pow(bitnum % 32);
                if(self.d[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.d[reg] = self.d[reg] & !mask;
            }
            1 => {//A register
                let mask = 1 << (bitnum % 32);
                if(self.a[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.a[reg] = self.a[reg] & (!mask as u32);
            }
            2 => {//address from A reg
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp & !mask) as u32, 1);
            }
            3 => {//A(n) with increment
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp & !mask)as u32, 1);
                self.a[reg] += 1;
            }
            4 => {//A(n) with decrement
                let mask = 1 << (bitnum % 7);
                self.a[reg] -= 1;
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.memory.mem_write(addr as usize, (temp & !mask)as u32, 1);
            }
            _ => {println!("invalid addressing mode for BCLRZ");}
        }

    }


    fn bset(&mut self){
        let bitnum = self.next_op() as u32;
        let reg = (self.op & 0b111) as usize;
        match (self.op >> 3) & 0b111 {//finding source
            0 => {//data register
                let mask = 1 << (bitnum % 32);
                if(self.d[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.d[reg] = self.d[reg] | mask;
            }
            1 => {//A register
                let mask = 1 << (bitnum % 32);
                if(self.a[reg] & mask) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.a[reg] = self.a[reg] | mask;
            }
            2 => {//address from A reg
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            3 => {//A(n) with increment
                let mask = 1 << (bitnum % 7);
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
                self.a[reg] += 1;
            }
            4 => { //A(n) with decrement
                let mask = 1 << (bitnum % 7)as u32;
                self.a[reg] -= 1;
                let addr = self.a[reg];
                let temp = self.memory.read_b(addr as usize);
                if(temp & mask as u8) == 0 {
                    self.sr = self.sr | 0b000000000000000000100;
                }
                else {
                    self.sr = self.sr & 0b111111111111111111011;
                }
            }
            _ => {println!("invalid addressing mode for op: BSETZ");}

        }
    }

    fn chk(&mut self){

    }

    fn movep(&mut self){
        let reg = ((self.op >> 9) & 0b111) as usize;
        let areg = (self.op & 0b111) as usize; //what address to use
        let addr = (self.next_op() as u32) + self.a[areg]; //the displacement to add
        if self.op & 0b10000000 != 0 {//FROM memory, TO d reg
            if self.op & 0b1000000 != 0 {//long
                self.d[reg] = self.memory.read_l(addr as usize);
            }
            else{//word
                self.d[reg] = by_byte(self.memory.read_w(addr as usize) as u32, self.d[reg], 2);
            }
        }
        else {//FROM d reg, TO memory
            if self.op & 0b1000000 != 0 {//long
                self.memory.mem_write(addr as usize, self.d[reg], 4);
            }
            else {
                self.memory.mem_write(addr as usize, self.d[reg], 2);
            }
        }

    }

    fn movem(&mut self){

    }

    fn addi(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        let reg: usize = (arg & 0b111) as usize;
        let mut mode = 0;
        match (arg >> 3) & 0b111 {
            0 => {
                let res = (self.d[reg] as i32 + temp as i32);
                match arg >> 6 & 0b11 {
                    0b00 => {
                        let check = (res & 0xff) as i8;
                        mode = 1;
                        if(res > 0xff){ 
                            self.sr = self.sr | 1;
                            self.sr = self.sr | 0b10;

                        }
                    }
                    0b01 => {
                        mode = 2;
                        let check = (res & 0xffff) as i16;
                    }
                    0b10 => {
                        let check = (res < temp as i32) | (res < self.d[reg] as i32);
                    }
                    _ => {}
                }
                self.d[reg] = by_byte(res as u32, self.d[reg], mode);
            }
            0b111 => {//fix this
            }
            _ => {}
        }
    }

    fn eori(&mut self) {
        let arg = self.op;
        let temp = self.next_op();
        if arg == 0x0A7c {
            return;
        }
        let mode = 2_u32.pow(((arg >> 6) & 0b11) as u32);
        let reg: usize = (arg & 0b111) as usize;
        match (arg >> 3) & 0b111 {
            0 => {
                self.d[reg] = by_byte((self.d[reg] ^ temp as u32), self.d[reg], mode);
            }
            0b111 => {
                let temp2 = self.next_op();
                let temp3 = self.memory.read_l(temp2 as usize) ^ (temp as u32);
                self.memory.mem_write(temp2 as usize, temp3, mode);
            }
            _ => {}
        }
    }

    fn cmpi(&mut self) {
        let arg = self.op;
        let mut mode = 0;
        let mut temp: u32 = 0;
        let mut temp2: u32 = 0;
        match ((arg >> 6) & 0b11) {
            // finding size of operation - mode is set according to the standard used by other ops
            0 => { 
                //byte
                mode = 1;
                temp = self.next_op() as u32;
            } 
            0b01 => {
                //word
                mode = 2; 
                temp = self.next_op() as u32;			
            } 
            0b10 => {
                //long
                mode = 4; 
                temp = self.next_l();
            } 
            _ => {}
        }

    }

    fn mov(&mut self) {
        let arg = self.op;
        let mut source: u32 = 0;
        let mut mode = 0;
        let reg = (arg & 0b111) as usize;
        match (arg >> 12) & 0b11 {
            0b01 => mode = 1,
            0b11 => mode = 2,
            _ => mode = 4,
        }
        match ((arg >> 3) & 0b111) {
            //finding source
            0 => {
                //source is a d register
                source = self.d[reg];
            }
            0b001 => {
                //source is an a reg
                source = self.a[reg];
            }
            0b010 => {
                //source is an address in an a reg
                source = self.memory.read_l(self.a[reg] as usize);
            }
            0b011 => {
                //source is an address in an a reg with post+
                source = self.memory.read_l(self.a[reg] as usize);
                self.a[reg] += mode;
            }
            0b100 => {
                //source is a reg with pre-
                self.a[reg] -= mode;
                source = self.memory.read_l(self.a[reg] as usize);
            }
            0b101 => {
                //source is an address with displacement
                let addr = (self.a[reg] + self.next_op() as u32) as usize;
                source = self.memory.read_l(addr);
            }
            0b110 => {
                //source is an address with index
                let temp = self.next_op();
                let mut addr = (self.a[reg] + (temp & 0xff) as u32) as usize;
                if (temp & 0x8000) != 0 {
                    addr += self.a[((temp >> 12) & 0b111) as usize] as usize;
                }
                else {
                    addr += self.d[((temp >> 12) & 0b111) as usize] as usize;
                }
                source = self.memory.read_l(addr);
            }
            0b111 => {
                //source is immediate
                match ((arg >> 12) & 0b11) {
                    0b10 => {
                        //longword immediate
                        source = ((self.next_op() as u32) << 16) + self.next_op() as u32;
                    }
                    //word and byte immediates are both coded as 16 bits, so they're
                    //treated as the same case here
                    _ => {
                        source = self.next_op() as u32;
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

    fn abcd(&mut self) {}
    fn and(&mut self) {}

    fn rte(&mut self) {
        let SR = self.memory.read_l(self.a[7] as usize);
        self.a[7] -= 4;
        let PC = self.memory.read_l(self.a[7] as usize);
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
        self.memory.mem_write(self.a[7] as usize, self.sr as u32, 4);
    }

    fn stop(&mut self) {
        if self.sr & 0x8000 != 0 {
            self.sr = self.next_op();
        } else {
            self.trap();
        }
    }

    fn rts(&mut self) {
        self.pc = self.memory.read_l(self.a[7] as usize);
        self.a[7] += 4;
    }

    fn unlk(&mut self) {
        let arg = self.op & 0b111;
        self.a[7] = self.a[arg as usize];
        let temp = self.memory.read_w(self.a[arg as usize] as usize) as u32;
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
    }

    fn trap(&mut self) {
        self.sr = self.sr | 0x8000;
        let arg = self.op & 0xf;
        println!("Call to trap # {}", arg);
    }

    fn trapv(&mut self) {

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
                        self.pc = ((self.next_op() as u32) << 16) 
                            + self.next_op() as u32;
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
        if (self.memory.read_b(temp) == 0) {
            self.memory.mem_write(temp, 80, 1);
        }
    }

    fn pea(&mut self) {

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
        let mut to_write: u32 = 0;
        if condition((self.op >> 8) & 0xf, self.sr) {
            to_write = 0xffffffff;
        }
        let temp = (self.op & 0b111) as usize;
        match (self.op >> 3) & 0b111 {
            0 => { // Dn - setting a d reg
                self.d[temp] = to_write;
            },
            1 => {} //invalid addressing mode for this op
            2 => {// (An) - setting memory from an a register
                let addr = self.a[temp] as usize;
                self.memory.mem_write(addr, to_write, 1);
            }
            3 => { // (An)+ - same as above but with increment
                let addr = self.a[temp] as usize;
                self.memory.mem_write(addr, to_write, 1);
                self.a[temp] += 1;
            }
            4 => { // -(An) - decrement
                self.a[temp] -= 1;
                let addr = self.a[temp] as usize;
                self.memory.mem_write(addr, to_write, 1);
            }
            5 => { // (An + d) - memory address with offset
                let mut addr = self.a[temp] as usize;
                addr += self.next_op() as usize;
                self.memory.mem_write(addr, to_write, 1);
            }
            6 => { // i need to figure out how indexing works

            }
            7 => {//some misc cases
                match temp {
                    0 => { //absolute near
                        let addr = self.next_op() as usize;
                        self.memory.mem_write(addr, to_write, 1);
                    }
                    1 => {
                        let addr = self.next_l() as usize;
                        self.memory.mem_write(addr, to_write, 1);
                    }
                    _ => {} //invalid addressing modes for this instruction

                }
            }
            _ => { /*shouldnt ever get here */ }
        }      
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
        let check = (self.op >> 8) & 0xf;
        if check == 1 {
            self.a[7] -= 4;
            let temp = self.memory.read_l(self.a[7] as usize);
            self.pc += offset as u32;
        }
        else if(condition((self.op >> 8) & 0xf, self.sr)){
            self.pc += offset as u32;
        }
    }

    fn dbcc(&mut self) {

    }

    fn div(&mut self) {

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
    
    fn add(&mut self) {

    }

    fn rot(&mut self) {
        println!("rot");
    }

    fn ls(&mut self) {
        println!("ls");
    }

    fn eor(&mut self) {

    }

    fn cmp(&mut self) {

    }

    fn sub(&mut self) {

    }

    fn or(&mut self){

    }

    fn sbcd(&mut self){

    }

    fn reset(&mut self){

    }

    fn next_l(&mut self) -> u32 {
        ((self.next_op() as u32) << 16) + (self.next_op() as u32)
    }

}

//SR: CVZNX

fn condition(code: u16, status: u16) -> bool{
    match code {
        0 => true,
        1 => false,
        0b0010 => {(status & 0b101) == 0}, //Higher
        0b0011 => {(status & 0b101) != 0}, //Lower or same
        0b0100 => {(status & 1) == 0}, //Carry clear
        0b0101 => {(status & 1) != 0}, //Carry set
        0b0110 => {(status & 0b100) == 0}, //Not equal
        0b0111 => {(status & 0b100) != 0 }, //Equal
        0b1000 => {(status & 0b10) == 0}, //Overflow clear
        0b1001 => {(status & 0b10) != 0}, //Overflow set
        0b1010 => {(status & 0b1000) == 0}, //Plus
        0b1011 => {(status & 0b1000) != 0}, //Minus
        0b1100 => {((status & 0b1010) == 0b1010) ||
            ((status & 0b1010) == 0b0000)}, //Greater/Equal
        0b1101 => {((status & 0b1010) == 0b1000) ||
            ((status & 0b1010) == 0b10)}, //Less than
        0b1110 => {(((status & 0b1010) == 0b1010) ||
                    ((status & 0b1010) == 0)) &&
            ((status & 0b100) == 0)}, //Greater
        0b1111 => {((status & 0b100) != 0) ||
            ((status & 0b1010) == 0b1000) ||
                ((status & 0b1010) == 0b10)}, //Less/Equal
        _ => true,
    }
}

fn by_byte(from: u32, to: u32, mode: u32) -> u32 {
    match mode {
        4 => return from, //long
        2 => {
            //word
            let temp = from & 0b00000000000000001111111111111111;
            let temp2 = to & 0b11111111111111110000000000000000;
            return temp + temp2;
        }
        1 => {
            //byte
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
    println!("{:#018b}", test.sr);
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

    pub fn read_b(&mut self, addr: usize) -> u8 {
        //The CPU leaves data in an array of u8s, and whatever needs to see
        //those values can request them here. This is a stopgap solution.
        self.m[addr]
    }

    pub fn read_w(&mut self, addr: usize) -> u16 {
        ((self.m[addr + 1] as u16) << 8) + (self.m[addr] as u16)
    }

    pub fn read_l(&mut self, addr: usize) -> u32 {
        ((self.read_w(addr + 2) as u32) << 16) + (self.read_w(addr as usize) as u32)
    }
}
