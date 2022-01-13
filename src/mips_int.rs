use std::collections::btree_map::Range;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use byteorder::{BigEndian, ReadBytesExt};

use crate::register::RegNames;
use crate::register::Register;
use regex::{Regex, Split};
use crate::{instruction, register};
use crate::instruction::OPName;

//https://www.cs.unibo.it/~solmi/teaching/arch_2002-2003/AssemblyLanguageProgDoc.pdf

#[derive(Debug)]
pub enum MipsError {
	UnknownInstruction(u32),
	SyntaxError(usize),
	MissingMain,
	InvalidMain,
	UnalignedBytes,
}

enum LoadingState {
	FileOpen,
	Data,
	Code
}

pub struct MipsInterpreter {
	stack: Vec<u8>,
	registers: [Register; 32],
	pc: Register,
	hi: Register,
	lo: Register,
	program: Vec<u32>,
	labels: HashMap<String, u32>,
}

/// Will remove a leading '$' if it exists,
/// and a trailing comma ',' if that also exists
///
/// # Arguments
///
/// * `inp` - The input string that will have its values trimmed
fn remove_symbols(inp: &str) -> String {
	let mut res;

	if inp.starts_with("$") {
		let (l, r) = inp.split_at(1); // l = "$"        r = rest
		res = String::from(r);
	} else {
		res = String::from(inp);
	}

	if res.ends_with(",") {
		let (l, r) = res.split_at(res.len()-1);
		res = String::from(l);
	}

	res
}

fn get_idx_from_reg_string(terms: &mut Split) -> Option<usize> {
	let Some(dest)  = terms.next() else { return None; };
	let dest = remove_symbols(dest);
	let dest = RegNames::register_align(&RegNames::str_to_enum(dest.as_str()));
	RegNames::idx_from_enum(&dest)
}

fn load_3_terms(terms: &mut Split) -> Option<u32> {
	let Some(dest) = get_idx_from_reg_string(terms) else {
		return None;
	};
	let mut opcode: u32 = (dest >> 16) as u32;
	let Some(first) = get_idx_from_reg_string(terms) else {
		return None;
	};
	opcode = opcode | (first >> 21) as u32;
	let Some(second) = get_idx_from_reg_string(terms) else {
		return None;
	};
	opcode = opcode | (second >> 26) as u32;
	Some(opcode)
}

fn load_2_terms_immediate(terms: &mut Split) -> Option<u32> {
	let Some(dest) = get_idx_from_reg_string(terms) else {
		return None;
	};
	let mut opcode: u32 = (dest >> 16) as u32;
	let Some(first) = get_idx_from_reg_string(terms) else {
		return None;
	};
	opcode = opcode | (first >> 21) as u32;
	None
}

impl MipsInterpreter {
	pub fn display_register(&self, reg: &RegNames) -> String {
		match reg {
			RegNames::PC => { self.pc.display_string() }
			RegNames::HI => { self.hi.display_string() }
			RegNames::LO => { self.lo.display_string() }
			_ => {
				if let Some(idx) = RegNames::idx_from_enum(reg) {
					return self.registers[idx].display_string()
				}
				String::from("ERROR DISPLAYING REGISTER")
			}
		}
	}

	fn inst_add(&mut self, inst: u32) {
		// $rd, $rs, $rt      =>     $rd = $rs + $rt

		// BIT LOCATIONS:
		//opcode	rs	    rt	    rd	    shift (shamt)	funct
		//6 bits	5 bits	5 bits	5 bits	5 bits	        6 bits
		let rs = (inst & 0b00000011111000000000000000000000) << 6;
		let rt = (inst & 0b00000000000111110000000000000000) << 11;
		let rd = (inst & 0b00000000000000001111100000000000) << 16;

		let rs_val = self.registers[rs as usize].get_u32();
		let rt_val = self.registers[rt as usize].get_u32();
		self.registers[rd as usize].set_u32( rs_val + rt_val );
	}

	fn inst_jump(&mut self, addr: u32) {
		//let index = self.labels[lbl];
		self.pc.set_u32( addr );
	}

	fn make_add(line: &str) -> Option<u32> {
		let mut opcode = instruction::OP_ADD;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_addi(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ADDI;
		Some(0)
	}

	fn make_addiu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ADDIU;
		Some(0)
	}

	fn make_addu(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_ADDU;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_sub(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_SUB;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_subu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SUBU;
		Some(0)
	}

	fn make_and(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_AND;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_andi(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ANDI;
		Some(0)
	}

	fn make_or(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_OR;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_ori(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ORI;
		Some(0)
	}

	fn make_xor(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_XOR;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_xori(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_XORI;
		Some(0)
	}

	fn make_nor(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_NOR;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_sll(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLL;
		Some(0)
	}

	fn make_srl(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SRL;
		Some(0)
	}

	fn make_sra(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SRA;
		Some(0)
	}

	fn make_sllv(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLLV;
		Some(0)
	}

	fn make_srlv(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SRLV;
		Some(0)
	}

	fn make_srav(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SRAV;
		Some(0)
	}

	fn make_slt(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLT;
		Some(0)
	}

	fn make_slti(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLTI;
		Some(0)
	}

	fn make_sltu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLTU;
		Some(0)
	}

	fn make_sltiu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SLTIU;
		Some(0)
	}

	fn make_mult(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_MULT;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_multu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_MULTU;
		Some(0)
	}

	fn make_div(line: &str) -> Option<u32>  {
		let mut opcode = instruction::OP_DIV;
		let inst_regex = Regex::new("[,\\s]+").unwrap();
		let mut terms = inst_regex.split(line);
		terms.next();
		let Some(regs) = load_3_terms(&mut terms) else {
			return None;
		};
		opcode = opcode | regs;

		Some(opcode)
	}

	fn make_divu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_DIVU;
		Some(0)
	}

	fn make_mfhi(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_MFHI;
		Some(0)
	}

	fn make_mthi(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_MTHI;
		Some(0)
	}

	fn make_mflo(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_MFLO;
		Some(0)
	}

	fn make_mtlo(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_MTLO;
		Some(0)
	}

	fn make_li(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ORI;
		Some(0)
	}

	fn make_lui(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_ORI;
		Some(0)
	}

	// memory related
	fn make_lw(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_LW;
		Some(0)
	}

	fn make_lh(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_LH;
		Some(0)
	}

	fn make_lhu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_LHU;
		Some(0)
	}

	fn make_lb(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_LB;
		Some(0)
	}

	fn make_lbu(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_LBU;
		Some(0)
	}

	fn make_sw(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SW;
		Some(0)
	}

	fn make_sh(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SH;
		Some(0)
	}

	fn make_sb(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SB;
		Some(0)
	}

	fn make_beq(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_BEQ;
		Some(0)
	}

	fn make_bne(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_BNE;
		Some(0)
	}

	fn make_jr(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_JR;
		Some(0)
	}

	fn make_j(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_J;
		Some(0)
	}

	fn make_jal(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_JAL;
		Some(0)
	}

	fn make_syscall(line: &str) -> Option<u32>  {
		let opcode = instruction::OP_SYSCALL;
		Some(0)
	}

	pub fn new() -> MipsInterpreter {
		MipsInterpreter {
			stack: vec![],
			registers: [
				Register::new(RegNames::R0),
				Register::new(RegNames::R1),
				Register::new(RegNames::R2),
				Register::new(RegNames::R3),
				Register::new(RegNames::R4),
				Register::new(RegNames::R5),
				Register::new(RegNames::R6),
				Register::new(RegNames::R7),
				Register::new(RegNames::R8),
				Register::new(RegNames::R9),
				Register::new(RegNames::R10),
				Register::new(RegNames::R11),
				Register::new(RegNames::R12),
				Register::new(RegNames::R13),
				Register::new(RegNames::R14),
				Register::new(RegNames::R15),
				Register::new(RegNames::R16),
				Register::new(RegNames::R17),
				Register::new(RegNames::R18),
				Register::new(RegNames::R19),
				Register::new(RegNames::R20),
				Register::new(RegNames::R21),
				Register::new(RegNames::R22),
				Register::new(RegNames::R23),
				Register::new(RegNames::R24),
				Register::new(RegNames::R25),
				Register::new(RegNames::R26),
				Register::new(RegNames::R27),
				Register::new(RegNames::R28),
				Register::new(RegNames::R29),
				Register::new(RegNames::R30),
				Register::new(RegNames::R31),
			],
			pc: Register::new(RegNames::PC),
			hi: Register::new(RegNames::HI),
			lo: Register::new(RegNames::LO),
			program: vec![],
			labels: HashMap::new(),
		}
	}

	fn reset(&mut self) {
		self.pc.set_u32( 0 );
		self.program = vec![];
		self.labels = HashMap::new();
	}

	fn get_opcode_from_instruction(code: u32) -> u32 {
		let first = code & instruction::OP_FIRST_CODE;
		if first != 0 {
			// We use the leading bits
			first
		} else {
			// We use the trailing bits
			code & instruction::OP_SECOND_CODE
		}
	}

	pub fn get_program_contents(&self) -> String {
		let mut s = String::with_capacity(self.program.len()*3);
		for line in self.program.iter() {
			s.push_str(format!("{}\n", line).as_str());
		}
		s
	}

	pub fn process_line(&mut self) -> Result<(), MipsError> {
		let inst= self.program[self.pc.get_u32() as usize];
		let opcode = MipsInterpreter::get_opcode_from_instruction(inst);
		match opcode {
			instruction::OP_ADD => { self.inst_add(inst); }
			instruction::OP_J => { self.inst_jump(inst); }
			_ => { return Err(MipsError::UnknownInstruction(inst)); }
		}
		self.pc.add_u32(4);

		Ok(())
	}

	fn read_val_or_immediate(vars: &mut HashMap<String, i32>,
							 labels: &mut HashMap<String, u32>,
							 terms: &mut Split) -> i32 {
		let val = terms.next().unwrap();
		match vars.entry(String::from(val)) {
			Entry::Occupied(e) => {
				*e.get() as i32
			}
			Entry::Vacant(_) => {
				match labels.entry(String::from(val)) {
					Entry::Occupied(e) => {
						*e.get() as i32
					}
					Entry::Vacant(_) => {
						val.parse::<i32>().unwrap()
					}
				}
			}
		}
	}

	fn load_zero_into_line(line: u32, idx: usize) -> u32 {
		match idx {
			0 => { line & 0b00000000111111111111111111111111 },
			1 => { line & 0b11111111000000001111111111111111 },
			2 => { line & 0b11111111111111110000000011111111 },
			3 => { line & 0b11111111111111111111111100000000 },
			_ => { 0 /* THIS SHOULD NEVER BE REACHED*/ }
		}
	}

	fn get_byte_segment_u32(line: u32, idx: usize) -> u8 {
		let res = match idx {
			0 => { (line & 0b11111111000000000000000000000000) >> 24 },
			1 => { (line & 0b00000000111111110000000000000000) >> 16 },
			2 => { (line & 0b00000000000000001111111100000000) >> 8 },
			3 => { (line & 0b00000000000000000000000011111111) },
			_ => { 0 /* THIS SHOULD NEVER BE REACHED*/ }
		} as u8;
		// rust refuses to handle this unless the data is named
		// so we're naming it just to return it
		res
	}

	fn get_byte_segment_u16(line: u16, idx: usize) -> u8 {
		let res = match idx {
			0 => { (line & 0b1111111100000000) >> 8 },
			1 => { (line & 0b0000000011111111)  },
			_ => { 0 /* THIS SHOULD NEVER BE REACHED*/ }
		} as u8;
		// rust refuses to handle this unless the data is named
		// so we're naming it just to return it
		res
	}

	fn add_byte_to_line(line: u32, pointer: u32, byte: u8) -> u32 {
		match pointer % 4 {
			0 => { line & (0b00000000111111111111111111111111 | (byte as u32) << 24) },
			1 => { line & (0b11111111000000001111111111111111 | (byte as u32) << 16) },
			2 => { line & (0b11111111111111110000000011111111 | (byte as u32) << 8) },
			3 => { line & (0b11111111111111111111111100000000 | byte as u32) },
			_ => { 0 /* Not actually reachable, but rust requires it. */ }
		}
	}

	pub fn load_program(&mut self, filename: &str) -> Result<(), MipsError> {
		println!("Loading {}", filename);
		let mut state = LoadingState::FileOpen;
		let mut variables: HashMap<String, i32> = HashMap::new();
		let mut labels: HashMap<String, u32> = HashMap::new();

		if let Ok(contents) = fs::read_to_string(filename) {
			self.reset(); // only reset if the file can be read/loaded
			let mut current_line: u32 = 0;		// line of meaningful text in the ASM

			// keeps track of every byte, not every line
			// this just rotates between 0-3 for the 32 bits per line
			let mut data_pointer: u32 = 0;
			for line in contents.lines() {
				println!("{} ({}): {}", data_pointer, data_pointer%4, line);
				let line = line.trim();
				if line.starts_with("#") || line.eq("") { continue; }

				// split on 'whitespace' and '='
				let inst_regex = Regex::new("[,\\s]+").unwrap();
				let assign_regex = Regex::new("[=\\s]+").unwrap();
				let label_regex = Regex::new("[A-Za-z]+:").unwrap();
				let quote_regex = Regex::new("\"").unwrap();
				if label_regex.is_match(line) {
					if line.starts_with("main:") {
						if data_pointer % 4 != 0 {
							return Err(MipsError::InvalidMain);
						}
					}
					labels.insert(String::from(line), data_pointer);
					continue;
				}

				// match within
				match state {

					LoadingState::FileOpen => {
						if line == ".data" {
							state = LoadingState::Data;
							continue;
						}
						let mut terms = assign_regex.split(line);
						let lbl = terms.next().unwrap().trim();
						let val = terms.next().unwrap().trim();
						variables.insert(lbl.parse().unwrap(), val.parse().unwrap());
					}

					LoadingState::Data => {
						if line == ".text" {
							state = LoadingState::Code;
							continue;
						}

						let mut terms = assign_regex.split(line);
						if line.starts_with(".align") {
							let _ = terms.next(); // the .assign keyword
							// the value is in terms of halfwords. I don't know why, it just is.
							let val: u32 = terms.next().unwrap().trim().parse().unwrap();
							if data_pointer % (2 * val) != 0 {
								self.program.push(current_line);
								data_pointer = (self.program.len() * 4) as u32;
								current_line = 0;
								continue;
							}
						} else if line.starts_with(".space") { /* 8 bits times the size */
							let _ = terms.next(); // the .assign keyword
							let val = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							for i in 0..val as usize {
								// loads in a SINGLE BYTE into where-ever the data_pointer says
								current_line = MipsInterpreter::load_zero_into_line(current_line, i%4);
								data_pointer += 1;
								if data_pointer % 4 == 0 {
									self.program.push(current_line);
									current_line = 0;
								}
							}
						} else if line.starts_with(".word") { /* 32 bits */
							let _ = terms.next(); // we can skip the ".word" at the start
							let val = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							for i in 0..4 {
								let b = MipsInterpreter::get_byte_segment_u32(val as u32, i);
								current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, b);
								data_pointer += 1;
								if data_pointer % 4 == 0 {
									self.program.push(current_line);
									current_line = 0;
								}
							}
						} else if line.starts_with(".halfword") { /* 16 bits */
							let _ = terms.next(); // we can skip the ".word" at the start
							let val = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							for i in 0..2 {
								let b = MipsInterpreter::get_byte_segment_u16(val as u16, i);
								current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, b);
								data_pointer += 1;
								if data_pointer % 4 == 0 {
									self.program.push(current_line);
									current_line = 0;
								}
							}
						} else if line.starts_with(".ascii") { /* 8 bits per character */
							let mut terms = quote_regex.split(line);
							let _ = terms.next(); // we can skip the .asciiz
							// and just look at the contents of the quote
							let contents = terms.next().unwrap();
							let mut escape_flag = false;
							for mut c in contents.chars() {
								if c == '\\' {
									escape_flag = true;
									continue;
								}
								if escape_flag && c == 'n' {
									c = '\n';
								}
								let b = c as u8;
								current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, b);
								data_pointer += 1;
								if data_pointer % 4 == 0 {
									self.program.push(current_line);
									current_line = 0;
								}
							}
						} else if line.starts_with(".asciiz") { /* 8 bits per character plus the empty */
							let mut terms = quote_regex.split(line);
							let _ = terms.next(); // we can skip the .asciiz
							// and just look at the contents of the quote
							let contents = terms.next().unwrap();
							for c in contents.chars() {
								let b = c.to_digit(10).unwrap() as u8;
								current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, b);
								data_pointer += 1;
								if data_pointer % 4 == 0 {
									self.program.push(current_line);
									current_line = 0;
								}
							}
							// 0 is the NULL character for ASCII
							current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, 0);
							data_pointer += 1;
							if data_pointer % 4 == 0 {
								self.program.push(current_line);
								current_line = 0;
							}
						} else if line.starts_with(".byte") { /* 8 bits */
							let _ = terms.next(); // we can skip the ".byte" at the start
							let val = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							//
							current_line = MipsInterpreter::add_byte_to_line(current_line, data_pointer, val as u8);
							data_pointer += 1;
							if data_pointer % 4 == 0 {
								self.program.push(current_line);
								current_line = 0;
							}
						} else if line.starts_with(".float") {
							// DISABLED
						} else if line.starts_with(".double") {
							// DISABLED
						}
					}

					LoadingState::Code => {
						let mut terms = inst_regex.split(line);
						if line.starts_with(".align") {
							let _ = terms.next(); // the .assign keyword
							// the value is in terms of halfwords. I don't know why, it just is.
							let val: u32 = terms.next().unwrap().trim().parse().unwrap();
							if data_pointer % (2 * val) != 0 {
								self.program.push(current_line);
								data_pointer = (self.program.len() * 4) as u32;
								current_line = 0;
								continue;
							}
						}

						let op = terms.next().unwrap();
						let opcode = OPName::from(op);
						let mut result = 0;
						let func = match opcode {
							instruction::OP_ADD => { MipsInterpreter::make_add }
							instruction::OP_ADDI => { MipsInterpreter::make_addi }
							instruction::OP_ADDIU => { MipsInterpreter::make_addiu }
							instruction::OP_ADDU => { MipsInterpreter::make_addu }
							instruction::OP_SUB => { MipsInterpreter::make_sub }
							instruction::OP_SUBU => { MipsInterpreter::make_subu }
							instruction::OP_AND => { MipsInterpreter::make_and }
							instruction::OP_ANDI => { MipsInterpreter::make_andi }
							instruction::OP_OR => { MipsInterpreter::make_or }
							instruction::OP_ORI => {
								if op == "li" {
									MipsInterpreter::make_li
								} else {
									MipsInterpreter::make_ori
								}
							}
							instruction::OP_XOR => { MipsInterpreter::make_xor }
							instruction::OP_XORI => { MipsInterpreter::make_xori }
							instruction::OP_NOR => { MipsInterpreter::make_nor }
							instruction::OP_SLL => { MipsInterpreter::make_sll }
							instruction::OP_SRL => { MipsInterpreter::make_srl }
							instruction::OP_SRA => { MipsInterpreter::make_sra }
							instruction::OP_SLLV => { MipsInterpreter::make_sllv }
							instruction::OP_SRLV => { MipsInterpreter::make_srlv }
							instruction::OP_SRAV => { MipsInterpreter::make_srav }
							instruction::OP_SLT => { MipsInterpreter::make_slt }
							instruction::OP_SLTI => { MipsInterpreter::make_slti }
							instruction::OP_SLTU => { MipsInterpreter::make_sltu }
							instruction::OP_SLTIU => { MipsInterpreter::make_sltiu }
							instruction::OP_MULT => { MipsInterpreter::make_mult }
							instruction::OP_MULTU => { MipsInterpreter::make_multu }
							instruction::OP_DIV => { MipsInterpreter::make_div }
							instruction::OP_DIVU => { MipsInterpreter::make_divu }
							instruction::OP_MFHI => { MipsInterpreter::make_mfhi }
							instruction::OP_MTHI => { MipsInterpreter::make_mthi }
							instruction::OP_MFLO => { MipsInterpreter::make_mflo }
							instruction::OP_MTLO => { MipsInterpreter::make_mtlo }
							instruction::OP_LUI => { MipsInterpreter::make_lui }
// memory related
							instruction::OP_LW => { MipsInterpreter::make_lw }
							instruction::OP_LH => { MipsInterpreter::make_lh }
							instruction::OP_LHU => { MipsInterpreter::make_lhu }
							instruction::OP_LB => { MipsInterpreter::make_lb }
							instruction::OP_LBU => { MipsInterpreter::make_lbu }
							instruction::OP_SW => { MipsInterpreter::make_sw }
							instruction::OP_SH => { MipsInterpreter::make_sh }
							instruction::OP_SB => { MipsInterpreter::make_sb }
// branching
							instruction::OP_BEQ => { MipsInterpreter::make_beq }
							instruction::OP_BNE => { MipsInterpreter::make_bne }
							instruction::OP_JR => { MipsInterpreter::make_jr }
							instruction::OP_J => { MipsInterpreter::make_j }
							instruction::OP_JAL => { MipsInterpreter::make_jal }
							instruction::OP_SYSCALL => { MipsInterpreter::make_syscall }
							_ => { return Err(MipsError::UnknownInstruction(opcode)); }
						};
						match func(line) {
							Some(res) => { result = res; }
							None => { return Err(MipsError::SyntaxError(current_line as usize)); }
						}

						self.program.push(result);
						data_pointer = (self.program.len() * 4) as u32;
					}
				}
			} // for each line
		}; // end if Ok(contents)

		match labels.entry(String::from("main")) {
			Entry::Occupied(v) => {
				self.pc.set_u32( *v.get() );
				Ok(())
			}
			Entry::Vacant(e) => { Err(MipsError::MissingMain) }
		}
	}
}
