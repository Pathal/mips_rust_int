use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::str::SplitWhitespace;
use byteorder::{BigEndian, ReadBytesExt};

use crate::mips_int::instruction::{OPName};
use crate::mips_int::register::RegNames;
use regex::{Regex, Split};

pub(crate) mod register;
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
		//
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
			return first
		} else {
			// We use the trailing bits
			code & instruction::OP_SECOND_CODE
		}
	}

	pub fn process_line(&mut self) -> Result<(), MipsError> {
		let inst;
		unsafe {
			inst = self.program[self.pc.value.uint as usize];
		}
		let opcode = MipsInterpreter::get_opcode_from_instruction(inst);
		match opcode {
			instruction::OP_ADD => { self.inst_add(inst); }
			instruction::OP_J => { self.inst_jump(inst); }
			_ => { return Err(MipsError::UnknownInstruction(inst)); }
		}
		// On a CPU, this would increase by 4
		// but we can keep things on a 32 bit level, instead of 8 (so increase by 1)
		unsafe {
			self.pc.value.uint += 1;
		}

		Ok(())
	}

	fn read_val_or_immediate(vars: &mut HashMap<String, i16>,
							 labels: &mut HashMap<String, usize>,
							 terms: &mut SplitWhitespace) -> i16 {
		let val = terms.next().unwrap();
		match vars.entry(String::from(val)) {
			Entry::Occupied(e) => {
				*e.get()
			}
			Entry::Vacant(_) => {
				match labels.entry(String::from(val)) {
					Entry::Occupied(e) => {
						*e.get() as i16
					}
					Entry::Vacant(_) => {
						val.parse::<i16>().unwrap()
					}
				}
			}
		}
	}

	pub fn load_program(&mut self, filename: &str) -> Result<(), MipsError> {
		println!("Loading {}", filename);
		let mut state = LoadingState::FileOpen;
		let mut variables: HashMap<String, i32> = HashMap::new();
		let mut labels: HashMap<String, u32> = HashMap::new();

		if let Ok(contents) = fs::read_to_string(filename) {
			self.reset(); // only reset if the file can be read/loaded
			let mut line_count : usize = 0;
			let mut data_pointer: usize = 0;	// keeps track of every byte, not every line
			let mut current_line: u32 = 0;		// line of meaningful text in the ASM
			for line in contents.lines() {
				println!("{}: {}", line_count, line);
				let line = line.trim();
				if line.starts_with("#") || line.eq("") { continue; }

				// split on whitespace and =
				let assign_regex = Regex::new("[=\\s]+").unwrap();
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

						if line.starts_with(".align") {
							let mut terms = assign_regex.split(line);
							let _ = terms.next(); // the .assign keyword
							let val: usize = terms.next().unwrap().trim().parse().unwrap(); // value to align by
							if data_pointer % (2 * val) != 0 {
								self.program.push(current_line);
								data_pointer = self.program.len()*4;
								current_line = 0;
								continue;
							}
						} else if line.starts_with(".space") {
							let mut terms = assign_regex.split(line);
							let _ = terms.next(); // the .assign keyword
							let val: usize = terms.next().unwrap().trim().parse().unwrap();
							for i in 0..val {

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
						self.program.push(current_line);
						current_line = 0;
					}
				}
				line_count += 1;
			} // for each line
		}; // end if Ok(contents)

		Ok(())
	}
}