use crate::mips_int::register;
use crate::mips_int::register::RegNames;

#[derive(Copy, Clone)]
pub enum OPName {
	VOID, // not a real function!
	ADD,
	ADDU,
	ADDI,
	ADDIU,
	SUB,
	SUBU,
	AND,
	ANDI,
	OR,
	ORI,
	XOR,
	XORI,
	NOR,
	SLL,
	SRL,
	SRA,
	SLLV,
	SRLV,
	SRAV,
	SLT,
	SLTI,
	SLTU,
	SLTIU,
	MULT,
	MULTU,
	DIV,
	DIVU,
	MFHI,
	MTHI,
	MFLO,
	MTLO,
	LI,
	LUI,
	// memory related
	LW,
	LH,
	LHU,
	LB,
	LBU,
	SW,
	SH,
	SB,
	// branching
	BEQ,
	BNE,
	JR,
	J,
	JAL,
	// Language/System
	SYSCALL
}

impl OPName {
	pub fn from(s: &str) -> OPName {
		match s.to_lowercase().as_str() {
			"add" => OPName::ADD,
			"addu" => OPName::ADDU,
			"addi" => OPName::ADDI,
			"addiu" => OPName::ADDIU,
			"sub" => OPName::SUB,
			"subu" => OPName::SUBU,
			"and" => OPName::AND,
			"andi" => OPName::ANDI,
			"or" => OPName::OR,
			"ori" => OPName::ORI,
			"xor" => OPName::XOR,
			"xori" => OPName::XORI,
			"nor" => OPName::NOR,
			"sll" => OPName::SLL,
			"srl" => OPName::SRL,
			"sra" => OPName::SRA,
			"sllv" => OPName::SLLV,
			"srlv" => OPName::SRLV,
			"srav" => OPName::SRAV,
			"slt" => OPName::SLT,
			"slti" => OPName::SLTI,
			"sltu" => OPName::SLTU,
			"sltiu" => OPName::SLTIU,
			"mult" => OPName::MULT,
			"multu" => OPName::MULTU,
			"div" => OPName::DIV,
			"divu" => OPName::DIVU,
			"mfhi" => OPName::MFHI,
			"mthi" => OPName::MTHI,
			"mflo" => OPName::MFLO,
			"mtlo" => OPName::MTLO,
			"li" => OPName::LI,
			"lui" => OPName::LUI,
			// memory related
			"lw" => OPName::LW,
			"lh" => OPName::LH,
			"lhu" => OPName::LHU,
			"lb" => OPName::LB,
			"lbu" => OPName::LBU,
			"sw" => OPName::SW,
			"sh" => OPName::SH,
			"sb" => OPName::SB,
			// branching
			"beq" => OPName::BEQ,
			"bne" => OPName::BNE,
			"jr" => OPName::JR,
			"j" => OPName::J,
			"jal" => OPName::JAL,
			"syscall" => OPName::SYSCALL,
			_ => OPName::VOID
		}
	}
}

#[derive(Copy, Clone)]
pub struct Instruction {
	pub op: OPName,
	pub rs: register::RegNames,
	pub rt: register::RegNames,
	pub rd: register::RegNames,
	//pub shamt: ??
	//pub funct: ??

	pub imm: i16,
	pub addr: usize,
	//pub lbl: String,
}

impl Instruction {
	pub fn new() -> Instruction {
		Instruction {
			op: OPName::VOID,
			rs: RegNames::ZERO,
			rt: RegNames::ZERO,
			rd: RegNames::ZERO,
			imm: 0,
			addr: 0,
			//lbl: "".parse().unwrap(),
		}
	}
}
