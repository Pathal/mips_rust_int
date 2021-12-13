use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::str::SplitWhitespace;

use crate::mips_int::instruction::{Instruction, OPName};
use crate::mips_int::register::RegNames;

pub(crate) mod register;
mod instruction;

#[derive(Debug)]
pub enum MipsError {
	UnknownInstruction,
	SyntaxError(usize),
}

pub struct MipsInterpreter {
	stack: Vec<u8>,
	registers: [register::Register; 32],
	pc: register::Register,
	hi: register::Register,
	lo: register::Register,
	program: Vec<instruction::Instruction>,
	labels: HashMap<String, i32>,
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

	fn inst_add(&mut self, inst: &instruction::Instruction) {
		//
	}

	fn inst_jump(&mut self, lbl: &usize) {
		//let index = self.labels[lbl];
		self.pc.value = *lbl as i32;
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
		self.pc.value = 0;
		self.program = vec![];
		self.labels = HashMap::new();
	}

	pub fn process_line(&mut self) -> Result<(), MipsError> {
		let inst = self.program[self.pc.value as usize].clone();
		match inst.op {
			OPName::ADD => self.inst_add(&inst),
			OPName::ADDU => {}
			OPName::ADDI => {}
			OPName::ADDIU => {}
			OPName::SUB => {}
			OPName::SUBU => {}
			OPName::AND => {}
			OPName::ANDI => {}
			OPName::OR => {}
			OPName::ORI => {}
			OPName::XOR => {}
			OPName::XORI => {}
			OPName::NOR => {}
			OPName::SLL => {}
			OPName::SRL => {}
			OPName::SRA => {}
			OPName::SLLV => {}
			OPName::SRLV => {}
			OPName::SRAV => {}
			OPName::SLT => {}
			OPName::SLTI => {}
			OPName::SLTU => {}
			OPName::SLTIU => {}
			OPName::MULT => {}
			OPName::MULTU => {}
			OPName::DIV => {}
			OPName::DIVU => {}
			OPName::MFHI => {}
			OPName::MTHI => {}
			OPName::MFLO => {}
			OPName::LI => {}
			OPName::LUI => {}
			OPName::LW => {}
			OPName::LH => {}
			OPName::LHU => {}
			OPName::LB => {}
			OPName::LBU => {}
			OPName::SW => {}
			OPName::SH => {}
			OPName::SB => {}
			OPName::BEQ => {}
			OPName::BNE => {}
			OPName::JR => {}
			OPName::J => self.inst_jump(&inst.addr),
			OPName::JAL => {}
			OPName::SYSCALL => {},
			_ => { return Err(MipsError::UnknownInstruction); }
		}

		// On a CPU, this would increase by 4
		// but we can keep things on a 32 bit level, instead of 8 (so increase by 1)
		self.pc.value += 1;

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
		let mut variables: HashMap<String, i16> = HashMap::new();
		let mut labels: HashMap<String, usize> = HashMap::new();
		if let Ok(contents) = fs::read_to_string(filename) {
			self.reset(); // only reset if the file can be read/loaded
			let mut line_count : usize = 0;
			for line in contents.lines() {
				println!("{}", line);
				let line = line.trim();
				if line == "" || line.starts_with("#") || line.starts_with(".data")
					|| line.starts_with(".align") || line.starts_with(".text")
				{ continue; }

				let mut terms = line.split_whitespace();
				if line.contains("=") {
					/* It's probably a variable */
					let k = terms.next().unwrap();
					terms.next();
					let v = terms.next().unwrap();
					variables.insert(String::from(k), v.parse::<i16>().unwrap());
				} else if line.ends_with(":") {
					/* It's probably a label */
					let l = terms.next().unwrap();
					let v = line_count;
					labels.insert(String::from(l), v);
					// We actually don't want to increment the line number,
					// which happens after these if blocks.
					// So we continue instead.
					continue;
				} else if line.starts_with(".") {
					/* TODO: Deal with this later */
				} else {
					/* It's probably an instruction */
					let mut inst = Instruction::new();
					let mut terms = line.split_whitespace();
					match terms.next() {
						None => { return Err(MipsError::SyntaxError(0)); }
						Some(v) => {
							inst.op = OPName::from(v);
						}
					}

					match inst.op {
						OPName::ADD => {
							// add $1,$2,$3 -> $1=$2+$3
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.rt = RegNames::str_to_enum(terms.next().unwrap());
							self.program.push(inst);
						}
						OPName::ADDU => {}
						OPName::ADDI => {
							//addi $1,$2,100
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.imm = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							self.program.push(inst);
						}
						OPName::ADDIU => {}
						OPName::SUB => {
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.rt = RegNames::str_to_enum(terms.next().unwrap());
							self.program.push(inst);
						}
						OPName::SUBU => {}
						OPName::AND => {
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.rt = RegNames::str_to_enum(terms.next().unwrap());
							self.program.push(inst);
						}
						OPName::ANDI => {
							//ANDI $1,$2,100
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.imm = MipsInterpreter::read_val_or_immediate(&mut variables, &mut labels, &mut terms);
							self.program.push(inst);
						}
						OPName::OR => {
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
							inst.rs = RegNames::str_to_enum(terms.next().unwrap());
							inst.rt = RegNames::str_to_enum(terms.next().unwrap());
							self.program.push(inst);
						}
						OPName::ORI => {}
						OPName::XOR => {}
						OPName::XORI => {}
						OPName::NOR => {}
						OPName::SLL => {}
						OPName::SRL => {}
						OPName::SRA => {}
						OPName::SLLV => {}
						OPName::SRLV => {}
						OPName::SRAV => {}
						OPName::SLT => {}
						OPName::SLTI => {}
						OPName::SLTU => {}
						OPName::SLTIU => {}
						OPName::MULT => {}
						OPName::MULTU => {}
						OPName::DIV => {}
						OPName::DIVU => {}
						OPName::MFHI => {}
						OPName::MTHI => {}
						OPName::MFLO => {}
						OPName::MTLO => {}
						OPName::LI => {}
						OPName::LUI => {}
						OPName::LW => {}
						OPName::LH => {}
						OPName::LHU => {}
						OPName::LB => {}
						OPName::LBU => {}
						OPName::SW => {}
						OPName::SH => {}
						OPName::SB => {}
						OPName::BEQ => {}
						OPName::BNE => {}
						OPName::JR => {
							inst.rd = RegNames::str_to_enum(terms.next().unwrap());
						}
						OPName::J => {
							if let Some(v) = terms.next() {
								match self.labels.entry(v.parse().unwrap()) {
									Entry::Occupied(val) => {inst.addr = *val.get() as usize;}
									Entry::Vacant(_) => {inst.addr = v.parse::<usize>().unwrap();}
								}
							}
						}
						OPName::JAL => {}
						OPName::SYSCALL => {}
						_ => { return Err(MipsError::SyntaxError(line_count)); }
					}
				}
				line_count += 1;
			}
		};

		Ok(())
	}
}