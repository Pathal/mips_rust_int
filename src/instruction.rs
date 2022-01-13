//https://opencores.org/projects/plasma/opcodes
//Notes: The immediate values are normally sign extended.
pub const OP_ADD: u32	= 0b00000000000000000000000000100000;
pub const OP_ADDI: u32	= 0b00100000000000000000000000000000;
pub const OP_ADDIU: u32	= 0b00100100000000000000000000000000;
pub const OP_ADDU: u32	= 0b00000000000000000000000000100001;
pub const OP_SUB: u32	= 0b00000000000000000000000000100010;
pub const OP_SUBU: u32	= 0b00000000000000000000000000100011;
pub const OP_AND: u32	= 0b00000000000000000000000000100100;
pub const OP_ANDI: u32	= 0b00110000000000000000000000000000;
pub const OP_OR: u32	= 0b00000000000000000000000000100101;
pub const OP_ORI: u32	= 0b00110100000000000000000000000000;
pub const OP_XOR: u32	= 0b00000000000000000000000000100110;
pub const OP_XORI: u32	= 0b00111000000000000000000000000000;
pub const OP_NOR: u32	= 0b00000000000000000000000000100111;
pub const OP_SLL: u32	= 0b00000000000000000000000000000000;
pub const OP_SRL: u32	= 0b00000000000000000000000000000010;
pub const OP_SRA: u32	= 0b00000000000000000000000000000011;
pub const OP_SLLV: u32	= 0b00000000000000000000000000000100;
pub const OP_SRLV: u32	= 0b00000000000000000000000000000110;
pub const OP_SRAV: u32	= 0b00000000000000000000000000000111;
pub const OP_SLT: u32	= 0b00000000000000000000000000101010;
pub const OP_SLTI: u32	= 0b00101000000000000000000000000000;
pub const OP_SLTU: u32	= 0b00000000000000000000000000101011;
pub const OP_SLTIU: u32 = 0b00101100000000000000000000000000;
pub const OP_MULT: u32	= 0b00000000000000000000000000011000;
pub const OP_MULTU: u32 = 0b00000000000000000000000000011001;
pub const OP_DIV: u32	= 0b00000000000000000000000000011010;
pub const OP_DIVU: u32	= 0b00000000000000000000000000011011;
pub const OP_MFHI: u32	= 0b00000000000000000000000000010000;
pub const OP_MTHI: u32	= 0b00000000000000000000000000010001;
pub const OP_MFLO: u32	= 0b00000000000000000000000000010010;
pub const OP_MTLO: u32	= 0b00000000000000000000000000010011;
pub const OP_LUI: u32	= 0b00111100000000000000000000000000;
// memory related
pub const OP_LW: u32	= 0b10001100000000000000000000000000;
pub const OP_LH: u32	= 0b10000100000000000000000000000000;
pub const OP_LHU: u32	= 0b10010100000000000000000000000000;
pub const OP_LB: u32	= 0b10000000000000000000000000000000;
pub const OP_LBU: u32	= 0b10010000000000000000000000000000;
pub const OP_SW: u32	= 0b10111000000000000000000000000000;
pub const OP_SH: u32	= 0b10101000000000000000000000000000;
pub const OP_SB: u32	= 0b10100000000000000000000000000000;
// branching
pub const OP_BEQ: u32	= 0b00010000000000000000000000000000;
pub const OP_BNE: u32	= 0b00010100000000000000000000000000;
pub const OP_JR: u32	= 0b00000000000000000000000000001000;
pub const OP_J: u32		= 0b00001000000000000000000000000000;
pub const OP_JAL: u32	= 0b00001100000000000000000000000000;
pub const OP_SYSCALL: u32 = 0b00000000000000000000000000001100;

// parsing numbers
pub const OP_FIRST_CODE: u32 = 0b11111100000000000000000000000000;
pub const OP_SECOND_CODE: u32 = 0b00000000000000000000000000111111;

#[allow(dead_code)]
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
	pub fn from(name: &str) -> u32 {
		match name {
			"add"	=> { OP_ADD },
			"addi"	=> { OP_ADDI },
			"addiu"	=> { OP_ADDIU },
			"addu"	=> { OP_ADDU },
			"sub"	=> { OP_SUB },
			"subu"	=> { OP_SUBU },
			"and"	=> { OP_AND },
			"andi"	=> { OP_ANDI },
			"or"	=> { OP_OR },
			"ori"	=> { OP_ORI },
			"xor"	=> { OP_XOR },
			"xori"	=> { OP_XORI },
			"nor"	=> { OP_NOR },
			"sll"	=> { OP_SLL },
			"srl"	=> { OP_SRL },
			"sra"	=> { OP_SRA },
			"sllv"	=> { OP_SLLV },
			"srlv"	=> { OP_SRLV },
			"srav"	=> { OP_SRAV },
			"slt"	=> { OP_SLT },
			"slti"	=> { OP_SLTI },
			"sltu"	=> { OP_SLTU },
			"sltiu" => { OP_SLTIU },
			"mult"	=> { OP_MULT },
			"multu" => { OP_MULTU },
			"div"	=> { OP_DIV },
			"divu"	=> { OP_DIVU },
			"mfhi"	=> { OP_MFHI },
			"mthi"	=> { OP_MTHI },
			"mflo"	=> { OP_MFLO },
			"mtlo"	=> { OP_MTLO },
			"li"	=> { OP_ORI }, // pseudo instruction
			"lui"	=> { OP_LUI },
			// memory related
			"lw"	=> { OP_LW },
			"lh"	=> { OP_LH },
			"lhu"	=> { OP_LHU },
			"lb"	=> { OP_LB },
			"lbu"	=> { OP_LBU },
			"sw"	=> { OP_SW },
			"sh"	=> { OP_SH },
			"sb"	=> { OP_SB },
			// branching
			"beq"	=> { OP_BEQ },
			"bne"	=> { OP_BNE },
			"jr"	=> { OP_JR },
			"j"		=> { OP_J },
			"jal"	=> { OP_JAL },
			"syscall" => { OP_SYSCALL },
			&_ => { 0 }
		}
	}
}
