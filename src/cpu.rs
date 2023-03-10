// for reference refer to
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use tinyrand::{Rand, StdRand};

pub struct Cpu {
    ram: [u8; 0xfff],
    pub vram: [[u8; 32]; 64],
    reg: [u8; 0x10], // registers
    i: u16,          // index register
    pc: u16,         // program counter
    stack: [u16; 0x10],
    sp: u8, // stack pointer
    dt: u8, // delay timer
    st: u8, // sound timer
    keypad: [u8; 0x10],
    rand: StdRand,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            ram: [0x0; 0xfff],
            vram: [[0x0; 32]; 64],
            reg: [0x0; 0x10],
            i: 0x0,
            pc: 0x200, // initial start adress once ROM is loaded
            stack: [0x0; 0x10],
            sp: 0x0,
            dt: 0x0,
            st: 0x0,
            keypad: [0x0; 0x10],
            rand: StdRand::default(),
        }
    }

    pub fn load_rom(&mut self, input: Vec<u8>) {
        let mut address: usize = 0x200;
        for byte in input.iter() {
            self.ram[address] = *byte;
            address += 1;
        }
    }

    pub fn step(&mut self) {
        // let inst: u8 = self.ram[self.pc as usize];
        let msb = self.ram[self.pc as usize] as u16;
        self.pc += 1;
        let lsb = self.ram[self.pc as usize] as u16;
        let mut inst: u16 = msb << 8;
        inst |= lsb;
        self.pc += 1;
        match inst & 0xf000 {
            0x0000 => match inst & 0x00ff {
                0x00e0 => self.op_00e0(inst),
                0x00ee => self.op_00ee(inst),
                _ => panic!("Instruction not valid: {}", inst),
            },
            0x1000 => self.op_1nnn(inst),
            0x2000 => self.op_2nnn(inst),
            0x3000 => self.op_3xkk(inst),
            0x4000 => self.op_4xkk(inst),
            0x5000 => self.op_5xy0(inst),
            0x6000 => self.op_6xkk(inst),
            0x7000 => self.op_7xkk(inst),
            0x8000 => match inst & 0x000f {
                0x0000 => self.op_8xy0(inst),
                0x0001 => self.op_8xy1(inst),
                0x0002 => self.op_8xy2(inst),
                0x0003 => self.op_8xy3(inst),
                0x0004 => self.op_8xy4(inst),
                0x0005 => self.op_8xy5(inst),
                0x0006 => self.op_8xy6(inst),
                0x0007 => self.op_8xy7(inst),
                0x000e => self.op_8xye(inst),
                _ => panic!("Instruction not valid: {}", inst),
            },
            0x9000 => self.op_9xy0(inst),
            0xa000 => self.op_annn(inst),
            0xb000 => self.op_bnnn(inst),
            0xc000 => self.op_cxkk(inst),
            0xd000 => self.op_dxyn(inst),
            0xe000 => match inst & 0x00ff {
                0x009e => self.op_ex9e(inst),
                0x00a1 => self.op_exa1(inst),
                _ => panic!("Instruction not valid: {}", inst),
            },
            0xf000 => match inst & 0x00ff {
                0x0007 => self.op_fx07(inst),
                0x000a => self.op_fx0a(inst),
                0x0015 => self.op_fx15(inst),
                0x0018 => self.op_fx18(inst),
                0x001e => self.op_fx1e(inst),
                0x0029 => self.op_fx29(inst),
                0x0033 => self.op_fx33(inst),
                0x0055 => self.op_fx55(inst),
                0x0065 => self.op_fx65(inst),
                _ => panic!("Instruction not valid: {}", inst),
            },
            _ => panic!("Instruction not valid: {}", inst),
        }
    }

    /* instructions */

    // 0nnn
    pub fn _op_0nnn(&mut self, _inst: u16) {
        // do nothing
    }

    // cls - clear the display
    // 00e0
    fn op_00e0(&mut self, _inst: u16) {
        self.vram = [[0x0; 32]; 64];
    }

    // ret - return from subroutine
    // 00ee
    fn op_00ee(&mut self, _inst: u16) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
        // self.pc = self.stack[self.sp as usize];
    }

    // jp - jump to location
    // 1nnn
    fn op_1nnn(&mut self, inst: u16) {
        self.pc = inst & 0x0fff;
    }

    // call - call subroutine
    // 2nnn
    fn op_2nnn(&mut self, inst: u16) {
        // need to subtract 2 to get the current instruction being run
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = inst & 0x0fff;
    }

    // se - skip next instruction if Vx = kk
    // 3xkk
    fn op_3xkk(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let val = (inst & 0x00ff) as u8;
        if self.reg[vx] == val {
            self.pc += 2;
        }
    }

    // sne - skip next instruction id vx != kk
    // 4xkk
    fn op_4xkk(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let val = (inst & 0x00ff) as u8;
        if self.reg[vx] != val {
            self.pc += 2;
        }
    }

    // reg se - skips the next if instruction is vx == vy
    // 5xy0
    fn op_5xy0(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        if self.reg[vx] == self.reg[vy] {
            self.pc += 2;
        }
    }

    // ld - put value kk into register vx
    // 6xkk
    fn op_6xkk(&mut self, inst: u16) {
        let register = ((inst & 0x0f00) >> 8) as usize;
        let value = (inst & 0x00ff) as u8;
        self.reg[register] = value;
    }

    // add - add vx by kk
    // 7xkk
    fn op_7xkk(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let value = (inst & 0x00ff) as u16;
        let x = self.reg[vx] as u16;
        let result = value + x;
        self.reg[vx] = result as u8;
    }

    // ld - store value in register y into register x
    // 8xy0
    pub fn op_8xy0(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        self.reg[vx] = self.reg[vy];
    }

    // or - bit wise or on registers vx and vy where the result goes into vx
    // 8xy1
    pub fn op_8xy1(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        self.reg[vx] |= self.reg[vy];
    }

    // and - bit wise an on rgisters vx and vy with the result going into vx
    // 8xy2
    pub fn op_8xy2(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        self.reg[vx] &= self.reg[vy];
    }

    // xor - bitwise exclusive or on registers vx and vy with the results going into vx
    // 8xy3
    pub fn op_8xy3(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        self.reg[vx] ^= self.reg[vy];
    }

    // add reg - add register contents of vx and vy
    // placing result into vx
    // setting carry flag
    // 8xy4
    pub fn op_8xy4(&mut self, inst: u16) {
        self.reg[0xf] = 0;
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        let sum: u16 = self.reg[vx] as u16 + self.reg[vy] as u16;
        if sum > 255 {
            self.reg[0xf] = 1;
        }
        self.reg[vx] = (sum & 0x00ff) as u8;
    }

    // sub reg - subtract registers contents vy from vx
    // result in vx
    // setting NOT borrow flag
    // 8xy5
    pub fn op_8xy5(&mut self, inst: u16) {
        self.reg[0xf] = 0;
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        if self.reg[vx] > self.reg[vy] {
            self.reg[0xf] = 1;
        }

        let x: i16 = self.reg[vx] as i16;
        let y: i16 = self.reg[vy] as i16;
        let z: i16 = x - y;
        // self.reg[vx as usize] -= self.reg[vy as usize];
        self.reg[vx as usize] = (z & 0x00ff) as u8;
    }

    // shr - shift right by 1
    // 8xy6
    pub fn op_8xy6(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.reg[0xf] = self.reg[vx] & 0x1;

        self.reg[vx as usize] >>= 1;
    }

    // subn
    // vx = vy - vx
    // 8xy7
    pub fn op_8xy7(&mut self, inst: u16) {
        self.reg[0xf] = 0;
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        if self.reg[vy] > self.reg[vx] {
            self.reg[0xf] = 1;
        }
        let x = self.reg[vx] as i16;
        let y = self.reg[vy] as i16;
        let z: i16 = y - x;
        // self.reg[vx as usize] = self.reg[vy as usize] - self.reg[vx as usize];
        self.reg[vx] = (z & 0x00ff) as u8;
    }

    // shl
    // 8xye
    pub fn op_8xye(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.reg[0xf] = (self.reg[vx] & 0x80) >> 7;
        self.reg[vx] <<= 1;
    }

    // sne - skip next inst if vx != vy
    // 9xy0
    pub fn op_9xy0(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        if self.reg[vx] != self.reg[vy] {
            self.pc += 2;
        }
    }

    // ls immed - the value of register i is set to nnn
    // annn
    pub fn op_annn(&mut self, inst: u16) {
        let val = inst & 0x0fff;
        self.i = val;
    }

    // jump reg - jump to location nnn + v0
    // bnnn
    pub fn op_bnnn(&mut self, inst: u16) {
        let val = (inst & 0x0fff) as u16;
        self.pc = self.reg[0] as u16 + val;
    }

    // rnd - set vx = random byte and kk
    // interpretor generate random number from 0 to 255, which is then ANDed with kk
    // cxkk
    pub fn op_cxkk(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let rand: u16 = self.rand.next_u16();
        let val: u16 = inst & 0x00ff;
        let result: u16 = rand & val;
        self.reg[vx] = (result & 0x00ff) as u8;
    }

    // drw
    // display n-byte sprite start at memory location I at (Vx, Vy), set VF = collision
    // dxyn
    pub fn op_dxyn(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let vy = ((inst & 0x00f0) >> 4) as usize;
        let height = (inst & 0x000f) as usize;

        let mut y_pos = (self.reg[vy] % 32) as usize;

        self.reg[0xf] = 0;
        for idx in 0..height {
            let mut x_pos = (self.reg[vx] % 64) as usize;
            let byte: u8 = self.ram[self.i as usize + idx];
            let mut mask: u8 = 0x80;
            let mut shift = 7;
            for _ in 0..8 {
                let pixel = (byte & mask) >> shift;
                mask >>= 1;
                shift -= 1;
                if self.vram[x_pos][y_pos] > 0 && pixel > 0 {
                    self.reg[0xf] = 1;
                }
                self.vram[x_pos][y_pos] ^= pixel;
                x_pos = (x_pos + 1) % 64;
            }
            y_pos = (y_pos + 1) % 32;
        }
    }

    // skp - skip next instruction if key with the calue of Vx is not pressed
    // ex9e
    pub fn op_ex9e(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let key = self.reg[vx] as usize;
        if self.keypad[key] > 0 {
            // research this
            self.pc += 2;
        }
    }

    // skpn
    // exa1
    pub fn op_exa1(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let key = self.reg[vx] as usize;
        if self.keypad[key] < 1 {
            self.pc += 2;
        }
    }

    // ld dt - set Vx = delay timer value
    // fx07
    pub fn op_fx07(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.reg[vx] = self.dt;
    }

    // ld keypress - wait for key press, stoe the value of the key in vx
    // fx0a
    pub fn op_fx0a(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        if self.keypad[0] > 0 {
            self.reg[vx] = 0;
        } else if self.keypad[1] > 0 {
            self.reg[vx] = 1;
        } else if self.keypad[2] > 0 {
            self.reg[vx] = 2;
        } else if self.keypad[3] > 0 {
            self.reg[vx] = 3;
        } else if self.keypad[4] > 0 {
            self.reg[vx] = 4;
        } else if self.keypad[5] > 0 {
            self.reg[vx] = 5;
        } else if self.keypad[6] > 0 {
            self.reg[vx] = 6;
        } else if self.keypad[7] > 0 {
            self.reg[vx] = 7;
        } else if self.keypad[8] > 0 {
            self.reg[vx] = 8;
        } else if self.keypad[9] > 0 {
            self.reg[vx] = 9;
        } else if self.keypad[10] > 0 {
            self.reg[vx] = 10;
        } else if self.keypad[11] > 0 {
            self.reg[vx] = 11;
        } else if self.keypad[12] > 0 {
            self.reg[vx] = 12;
        } else if self.keypad[13] > 0 {
            self.reg[vx] = 13;
        } else if self.keypad[14] > 0 {
            self.reg[vx] = 14;
        } else if self.keypad[15] > 0 {
            self.reg[vx] = 15;
        } else {
            self.pc -= 2;
        }
    }

    // ld set delay - delay time = vx
    // fx15
    pub fn op_fx15(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.dt = self.reg[vx];
    }

    // ld st, set sound timer = vx
    // fx18
    pub fn op_fx18(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.st = self.reg[vx];
    }

    // add i, vx
    // i = i + vx
    // fx1e
    pub fn op_fx1e(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        self.i += self.reg[vx] as u16;
    }

    // ld f, vx
    // set i = location of sprite for digit vx
    // fx29
    pub fn op_fx29(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let digit = self.reg[vx] as u16;
        // 100 is supposed to be fontsize address
        self.i = 100 + (5 * digit);
    }

    // ld b, vx
    // store BCD representation of Vx in memory locations I, I + 1, I + 2
    // fx33
    pub fn op_fx33(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        let mut val = self.reg[vx];
        self.ram[self.i as usize + 2] = val % 10;
        val /= 10;
        self.ram[self.i as usize + 1] = val % 10;
        val /= 10;
        self.ram[self.i as usize] = val % 10;
    }

    // ld [i], vx
    // store contents of registers v0 trhough vx to memory starting at index location
    // fx55
    pub fn op_fx55(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        for idx in 0..vx {
            self.ram[self.i as usize + idx] = self.reg[idx as usize];
        }
    }

    // ld vx, [i]
    // load contents into registers from ram
    // fx65
    pub fn op_fx65(&mut self, inst: u16) {
        let vx = ((inst & 0x0f00) >> 8) as usize;
        for idx in 0..vx {
            self.reg[idx as usize] = self.ram[self.i as usize + idx];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_00e0() {
        let mut cpu = Cpu::new();
        cpu.vram = [[0x1; 32]; 64];
        for i in 0..cpu.vram.len() {
            for j in 0..cpu.vram[0].len() {
                assert_eq!(cpu.vram[i][j], 0x1);
            }
        }
        cpu.op_00e0(0x00e0);
        for i in 0..cpu.vram.len() {
            for j in 0..cpu.vram[0].len() {
                assert_eq!(cpu.vram[i][j], 0x0);
            }
        }
    }

    #[test]
    fn test_op_1nnn() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [0x12, 0x00, 0x00, 0xe0].to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_op_2nnn_and_00ee() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            0x22, 0x04, // call subroutine at address 0x204
            0x00, 0xe0, // fill for address 0x202 and 0x203
            0x00, 0xee, // instruction to return
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[cpu.sp as usize], 0x202);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_op_3xkk() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // skp if equal instruction
            0x31, 0x11, // instruction to skip,, filler instruction
            0x00, 0x00, // 0x204
            // instruction to execute next
            0x00, 0x00,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.reg[1] = 0x11;
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_op_4xkk() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // addres 0x200
            // ship if not equal instriction
            0x41, 0x11, // address 0x202
            // instruction skipped
            0x00, 0x00,
            // address 0x204
            // instruction ran if equal
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.reg[1] = 0x12;
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_op_5xy0() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // check values in register 1 and 2
            0x51, 0x20, // address 0x202
            // instruction skipped
            0x00, 0x00, // address 0x204
            // instruction to be executed next
            0x12, 00,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.reg[1] = 1;
        cpu.reg[2] = 1;
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_op_6xkk() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value into register 1
            0x61, 0xff, // address 0x202
            // filler
            0x12, 0x00,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0xff);
    }

    #[test]
    fn test_op_7xkk() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 1 into register 1
            0x61, 0x01, // address 0x202
            // vx = vx + kk
            0x71, 0x01,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x02);
    }

    #[test]
    fn test_op_7xkk_overflow() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 1 into register 1
            0x61, 0x02, // address 0x202
            // vx = vx + kk
            0x71, 0xff,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x01);
    }

    #[test]
    fn test_op_8xy0() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0xff into register 2
            0x62, 0xff, // address 0x202
            // load value in register 2 to register 1
            0x81, 0x20,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[2], 0x00);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[2], 0xff);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0xff);
        assert_eq!(cpu.reg[1], 0xff);
    }

    #[test]
    fn test_op_8xy1() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load 0x02 into register 2
            0x62, 0x02, // address 204
            // reg[x] ^ reg[y] = reg[x]
            0x81, 0x21,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], (0x01 | 0x02));
        assert_eq!(cpu.reg[2], 0x02);
    }

    #[test]
    fn test_op_8xy2() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load value 0x01 into register 2
            0x62, 0x01, // address 0x204
            // vx & vy = vx
            0x81, 0x22,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], (0x01 & 0x01));
        assert_eq!(cpu.reg[2], 0x01);
    }

    #[test]
    fn test_op_8xy3() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load value 0x01 into register 2
            0x62, 0x01, // address 0x204
            // vx ^ vy = vx
            0x81, 0x23,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[2], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], (0x01 ^ 0x01));
        assert_eq!(cpu.reg[2], 0x01);
    }

    #[test]
    fn test_op_8xy4_carry_flag() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load 0xff into register 2
            0x62, 0xff,
            // address 0x204
            // perform instruction 8xy4 on registers 1 and 2
            0x81, 0x24,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0xff);
        cpu.step();
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[0xf], 0x01);
    }

    #[test]
    fn test_op_8xy4_no_carry_flag() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load 0xff into register 2
            0x62, 0x08,
            // address 0x204
            // perform instruction 8xy4 on registers 1 and 2
            0x81, 0x24,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x08);
        cpu.step();
        assert_eq!(cpu.reg[1], 0x09);
        assert_eq!(cpu.reg[0xf], 0x00);
    }

    #[test]
    fn test_op_8xy5_carry_flag() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x02 into register 1
            0x61, 0x02, // address 0x202
            // load 0x01 into register 2
            0x62, 0x01, // address 0x204
            // perform 8xy5 on registers 1 and 2
            0x81, 0x25,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x01);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[0xf], 0x01);
    }

    #[test]
    fn test_op_8xy5_no_carry_flag() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x02 into register 1
            0x61, 0x02, // address 0x202
            // load 0x01 into register 2
            0x62, 0x03, // address 0x204
            // perform 8xy5 on registers 1 and 2
            0x81, 0x25,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x03);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.reg[1], 0xff);
        assert_eq!(cpu.reg[0xf], 0x00);
    }

    #[test]
    fn test_op_8xy6_no_set_vf() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x04 into register 1
            0x61, 0x04, // address 0x202
            // perform shr 8xy6 operation
            0x81, 0x06,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x04);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x02);
        assert_eq!(cpu.reg[0xf], 0x00);
    }

    #[test]
    fn test_op_8xy6_set_vf() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x03 into register 1
            0x61, 0x03, // address 0x202
            // perform shr 8xy6 operation
            0x81, 0x06,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x03);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x01);
        assert_eq!(cpu.reg[0xf], 0x01);
    }

    #[test]
    fn test_op_8xy7_no_borrow() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load value 0x05 into register 2
            0x62, 0x05, // address 0x204
            // perform 8xy7 on registers 1 and 2
            0x81, 0x27,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x05);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], 0x04);
        assert_eq!(cpu.reg[0xf], 0x01);
    }

    #[test]
    fn test_op_8xy7_borrow() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x06 into register 1
            0x61, 0x06, // address 0x202
            // load value 0x05 into register 2
            0x62, 0x05, // address 0x204
            // perform 8xy7 on registers 1 and 2
            0x81, 0x27,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x06);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x05);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], 0xff);
        assert_eq!(cpu.reg[0xf], 0x00);
    }

    #[test]
    fn test_op_8xye_no_set_vf() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x4 into register 1
            0x61, 0x04, // address 0x202
            // shift left register 1
            0x81, 0x0e,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x04);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x08);
        assert_eq!(cpu.reg[0xf], 0x00);
    }

    #[test]
    fn test_op_8xye_set_vf() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load value 0x81 into register 1
            0x61, 0x81, // address 0x202
            // shift left register 1
            0x81, 0x0e,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[0xf], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x81);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[1], 0x02);
        assert_eq!(cpu.reg[0xf], 0x01);
    }

    #[test]
    fn test_op_9xy0_skip_instruction() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load 0x02 into register 2
            0x62, 0x02,
            // address 0x204
            // perform equality check on register 1 and 2
            0x91, 0x20, // address 0x206
            // instruction to skip
            0x61, 0x04, // address 0x208
            // load 0x02 into register 1
            0x61, 0x02,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x208);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x20a);
        assert_eq!(cpu.reg[1], 0x02);
    }

    #[test]
    fn test_op_9xy0_no_skip_instruction() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x01 into register 1
            0x61, 0x01, // address 0x202
            // load 0x01 into register 2
            0x62, 0x01,
            // address 0x204
            // perform equality check on register 1 and 2
            0x91, 0x20, // address 0x206
            // instruction to skip
            0x61, 0x04, // address 0x208
            // load 0x02 into register 1
            0x61, 0x02,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        assert_eq!(cpu.reg[2], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.reg[2], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        assert_eq!(cpu.reg[1], 0x01);
        cpu.step();
        assert_eq!(cpu.pc, 0x208);
        assert_eq!(cpu.reg[1], 0x04);
    }

    #[test]
    fn test_op_annn() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // addres 0x200
            // perform op annn
            0xa1, 0x11,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.i, 0x0000);
        cpu.step();
        assert_eq!(cpu.i, 0x0111);
    }

    #[test]
    fn test_op_bnnn() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load 0x02 into register 0
            0x60, 0x02, // address 0x202
            // jump to v0 + nnn,
            0xb2, 0x04,
            // address 0x204
            // load register 1 with 0x2, should be jumped overtime
            0x61, 0x02,
            // address 0x206
            // load register 1 with 0x4, should be jumped to
            0x61, 0x04,
        ]
        .to_vec();
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[0], 0x00);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[0], 0x02);
        cpu.step();
        assert_eq!(cpu.pc, 0x206);
        cpu.step();
        assert_eq!(cpu.pc, 0x208);
        assert_eq!(cpu.reg[1], 0x04);
    }

    #[test]
    fn test_op_fx07() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // set vx to dt
            0xf1, 0x07,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.dt = 0x01;
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.reg[1], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.reg[1], 0x01);
    }

    #[test]
    fn test_op_fx15() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // set vx to dt
            0xf1, 0x15,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.reg[1] = 0x01;
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.dt, 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.dt, 0x01);
    }

    #[test]
    fn test_op_fx18() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // set vx to dt
            0xf1, 0x18,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.reg[1] = 0x01;
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.st, 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.st, 0x01);
    }

    #[test]
    fn test_op_fx1e() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // add contents of vx to index register
            0xf1, 0x1e,
        ]
        .to_vec();
        cpu.load_rom(rom);
        cpu.i = 0x01;
        cpu.reg[1] = 0x01;
        assert_eq!(cpu.pc, 0x200);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.i, 0x02);
    }

    // ld f, vx
    // set i = location of sprite for digit vx
    // fx29
    // pub fn op_fx29(&mut self, inst: u16) {
    //     let vx = ((inst & 0x0f00) >> 8) as usize;
    //     let digit = self.reg[vx] as u16;
    //     // 100 is supposed to be fontsize address
    //     self.i = 100 + (5 * digit);
    // }

    // ld b, vx
    // store BCD representation of Vx in memory locations I, I + 1, I + 2
    // fx33
    // pub fn op_fx33(&mut self, inst: u16) {
    //     let vx = ((inst & 0x0f00) >> 8) as usize;
    //     let mut val = self.reg[vx];
    //     self.ram[self.i as usize + 2] = val % 10;
    //     val /= 10;
    //     self.ram[self.i as usize + 1] = val % 10;
    //     val /= 10;
    //     self.ram[self.i as usize] = val % 10;
    // }

    #[test]
    fn test_op_fx55() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load contents of all register into ram
            0xff, 0x55,
        ]
        .to_vec();
        cpu.reg[0] = 1;
        cpu.reg[1] = 1;
        cpu.reg[2] = 1;
        cpu.reg[3] = 1;
        cpu.reg[4] = 1;
        cpu.reg[5] = 1;
        cpu.reg[6] = 1;
        cpu.reg[7] = 1;
        cpu.reg[8] = 1;
        cpu.reg[9] = 1;
        cpu.reg[0xa] = 1;
        cpu.reg[0xb] = 1;
        cpu.reg[0xc] = 1;
        cpu.reg[0xd] = 1;
        cpu.reg[0xe] = 1;
        cpu.reg[0xf] = 1;
        cpu.i = 0x000;
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.ram[cpu.i as usize], 0x00);
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        for i in 0..15 {
            assert_eq!(cpu.ram[cpu.i as usize + i], 0x01);
        }
    }

    #[test]
    fn test_op_fx65() {
        let mut cpu = Cpu::new();
        let rom: Vec<u8> = [
            // address 0x200
            // load contents from memory to registers
            0xff, 0x65,
        ]
        .to_vec();
        cpu.ram[0] = 1;
        cpu.ram[1] = 1;
        cpu.ram[2] = 1;
        cpu.ram[3] = 1;
        cpu.ram[4] = 1;
        cpu.ram[5] = 1;
        cpu.ram[6] = 1;
        cpu.ram[7] = 1;
        cpu.ram[8] = 1;
        cpu.ram[9] = 1;
        cpu.ram[0xa] = 1;
        cpu.ram[0xb] = 1;
        cpu.ram[0xc] = 1;
        cpu.ram[0xd] = 1;
        cpu.ram[0xe] = 1;
        cpu.ram[0xf] = 1;
        cpu.load_rom(rom);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.i, 0x000);
        for i in 0..15 {
            assert_eq!(cpu.reg[i], 0x00);
        }
        cpu.step();
        assert_eq!(cpu.pc, 0x202);
        for i in 0..15 {
            assert_eq!(cpu.reg[i], 0x01);
        }
    }
}
