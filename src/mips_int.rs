use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use byteorder::{BigEndian, ReadBytesExt};

use crate::mips_int::register::RegNames;
use regex::{Regex, Split};

pub mod register;
mod instruction;

//https://www.cs.unibo.it/~solmi/teaching/arch_2002-2003/AssemblyLanguageProgDoc.pdf

#[derive(Debug)]
pub enum MipsError {
	UnknownInstruction(u32),
	SyntaxError(usize),
}

enum LoadingState {
	FileOpen,
	Data,
	Code
}

pub struct MipsInterpreter {
	stack: Vec<u8>,
	registers: [register::Register; 32],
	pc: register::Register,
	hi: register::Register,
	lo: register::Register,
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
		self.registers[rd as usize].value.uint = rs_val + rt_val;
	}

	fn inst_jump(&mut self, addr: u32) {
		//let index = self.labels[lbl];
		self.pc.value.uint = addr;
	}

	pub fn new() -> MipsInterpreter {
		MipsInterpreter {
			stack: vec![],
			registers: [
				register::Register::new(RegNames::R0),
				register::Register::new(RegNames::R1),
				register::Register::new(RegNames::R2),
				register::Register::new(RegNames::R3),
				register::Register::new(RegNames::R4),
				register::Register::new(RegNames::R5),
				register::Register::new(RegNames::R6),
				register::Register::new(RegNames::R7),
				register::Register::new(RegNames::R8),
				register::Register::new(RegNames::R9),
				register::Register::new(RegNames::R10),
				register::Register::new(RegNames::R11),
				register::Register::new(RegNames::R12),
				register::Register::new(RegNames::R13),
				register::Register::new(RegNames::R14),
				register::Register::new(RegNames::R15),
				register::Register::new(RegNames::R16),
				register::Register::new(RegNames::R17),
				register::Register::new(RegNames::R18),
				register::Register::new(RegNames::R19),
				register::Register::new(RegNames::R20),
				register::Register::new(RegNames::R21),
				register::Register::new(RegNames::R22),
				register::Register::new(RegNames::R23),
				register::Register::new(RegNames::R24),
				register::Register::new(RegNames::R25),
				register::Register::new(RegNames::R26),
				register::Register::new(RegNames::R27),
				register::Register::new(RegNames::R28),
				register::Register::new(RegNames::R29),
				register::Register::new(RegNames::R30),
				register::Register::new(RegNames::R31),
			],
			pc: register::Register::new(RegNames::PC),
			hi: register::Register::new(RegNames::HI),
			lo: register::Register::new(RegNames::LO),
			program: vec![],
			labels: HashMap::new(),
		}
	}

	fn reset(&mut self) {
		self.pc.value.uint = 0;
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
		unsafe {
			// because this mutation also reads from the union, we must wrap in unsafe
			self.pc.value.uint += 4;
		}

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

						if label_regex.is_match(line) {
							labels.insert(String::from(line), data_pointer);
						} else if line.starts_with(".align") {
							let mut terms = assign_regex.split(line);
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
							let mut terms = assign_regex.split(line);
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
							//
						} else if line.starts_with(".halfword") {
							//
						} else if line.starts_with(".ascii") {
							//
						} else if line.starts_with(".asciiz") {
							//
						} else if line.starts_with(".byte") {
							//
						} else if line.starts_with(".float") {
							//
						} else if line.starts_with(".double") {
							//
						}
					}

					LoadingState::Code => {
						if label_regex.is_match(line) {
							labels.insert(String::from(line), data_pointer);
						} else {
							self.program.push(current_line);
							current_line = 0;
						}
					}
				}
			} // for each line
		}; // end if Ok(contents)

		self.pc.value.uint = *labels.get("main").unwrap();

		Ok(())
	}
}