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

//https://www.cs.unibo.it/~solmi/teaching/arch_2002-2003/AssemblyLanguageProgDoc.pdf

#[derive(Debug)]
pub enum MipsError {
	UnknownInstruction(u32),
	SyntaxError(usize),
	MissingMain,
	InvalidMain,
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

impl MipsInterpreter {
	pub fn display_register(&self, reg: &RegNames) -> String {
		match reg {
			RegNames::PC => { self.pc.display_string() }
			RegNames::HI => { self.hi.display_string() }
			RegNames::LO => { self.lo.display_string() }
			_ => {
				let idx = RegNames::idx_from_enum(reg);
				self.registers[idx].display_string()
			}
		}
	}

	fn inst_add(&mut self, inst: u32) {
		//rd = rs + rt
		let rs = (inst & 0b00000011111000000000000000000000) >> 21;
		let rt = (inst & 0b00000000000111110000000000000000) >> 16;
		let rd = (inst & 0b00000000000000001111100000000000) >> 11;

		let rs_val = self.registers[rs as usize].get_u32();
		let rt_val = self.registers[rt as usize].get_u32();
		self.registers[rd as usize].set_u32( rs_val + rt_val );
	}

	fn inst_jump(&mut self, addr: u32) {
		//let index = self.labels[lbl];
		self.pc.set_u32( addr );
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
						} else if line.starts_with(".space") {
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
						} else if line.starts_with(".word") {
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
						} else if line.starts_with(".halfword") {
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
						} else if line.starts_with(".ascii") {
							//
						} else if line.starts_with(".asciiz") {
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
						} else if line.starts_with(".byte") {
							//
						} else if line.starts_with(".float") {
							//
						} else if line.starts_with(".double") {
							//
						}
					}

					LoadingState::Code => {
						self.program.push(current_line);
						current_line = 0;
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
