#[derive(Copy, Clone)]
pub enum RegNames {
	R0,
	R1,
	R2,
	R3,
	R4,
	R5,
	R6,
	R7,
	R8,
	R9,
	R10,
	R11,
	R12,
	R13,
	R14,
	R15,
	R16,
	R17,
	R18,
	R19,
	R20,
	R21,
	R22,
	R23,
	R24,
	R25,
	R26,
	R27,
	R28,
	R29,
	R30,
	R31,

	ZERO,	// 0:	hardwired to 0
	At,		// 1:	reserved for assembler
	V0,		// 2-3:	fn return values
	V1,
	A0,		// 4-7:	fn arguments
	A1,
	A2,
	A3,
	T0,		// 8-15:fn temporary regs
	T1,
	T2,
	T3,
	T4,
	T5,
	T6,
	T7,
	T8,		// 24-25:don't remember why these are separate
	T9,
	S0,		// 16-23:saved regs, must save values on stack
	S1,		// 		 and restore when returning from fn
	S2,
	S3,
	S4,
	S5,
	S6,
	S7,
	K0,		// 26-27:reserved for OS kernel
	K1,
	GP,		// 28:	 global vars in memory
	SP,		// 29:	 stack pointer
	FP,		// 30:	 frame pointer
	RA,		// 31:	 return address for fn

	PC,		// Program Counter
	HI,		// Mul/Div Hi
	LO,		// Mul/Div Lo
}

impl RegNames {
	pub fn str_to_enum(s: &str) -> RegNames {
		match s {
			"R1" => RegNames::R1,
			"R2" => RegNames::R2,
			"R3" => RegNames::R3,
			"R4" => RegNames::R4,
			"R5" => RegNames::R5,
			"R6" => RegNames::R6,
			"R7" => RegNames::R7,
			"R8" => RegNames::R8,
			"R9" => RegNames::R9,
			"R10" => RegNames::R10,
			"R11" => RegNames::R11,
			"R12" => RegNames::R12,
			"R13" => RegNames::R13,
			"R14" => RegNames::R14,
			"R15" => RegNames::R15,
			"R16" => RegNames::R16,
			"R17" => RegNames::R17,
			"R18" => RegNames::R18,
			"R19" => RegNames::R19,
			"R20" => RegNames::R20,
			"R21" => RegNames::R21,
			"R22" => RegNames::R22,
			"R23" => RegNames::R23,
			"R24" => RegNames::R24,
			"R25" => RegNames::R25,
			"R26" => RegNames::R26,
			"R27" => RegNames::R27,
			"R28" => RegNames::R28,
			"R29" => RegNames::R29,
			"R30" => RegNames::R30,
			"R31" => RegNames::R31,
			// now for the special names
			"T0" => RegNames::T0,
			"T1" => RegNames::T1,
			"T2" => RegNames::T2,
			"T3" => RegNames::T3,
			"T4" => RegNames::T4,
			"T5" => RegNames::T5,
			"T6" => RegNames::T6,
			"T7" => RegNames::T7,
			"T8" => RegNames::T8,
			"T9" => RegNames::T9,

			"S0" => RegNames::S0,
			"S1" => RegNames::S1,
			"S2" => RegNames::S2,
			"S3" => RegNames::S3,
			"S4" => RegNames::S4,
			"S5" => RegNames::S5,
			"S6" => RegNames::S6,
			"S7" => RegNames::S7,

			"HI" => RegNames::HI,
			"LO" => RegNames::LO,
			"PC" => RegNames::PC,

			_ => RegNames::ZERO
		}
	}

	pub fn to_str(n: &RegNames) -> String {
		match n {
			RegNames::R0 => { String::from("R0/Z") }
			RegNames::R1 => { String::from("R1/At") }
			RegNames::R2 => { String::from("R2/V0") }
			RegNames::R3 => { String::from("R3/V1") }
			RegNames::R4 => { String::from("R4/A0") }
			RegNames::R5 => { String::from("R5/A1") }
			RegNames::R6 => { String::from("R6/A2") }
			RegNames::R7 => { String::from("R7/A3") }

			RegNames::R8  => { String::from("R8/T0") }
			RegNames::R9  => { String::from("R9/T1") }
			RegNames::R10 => { String::from("R10/T2") }
			RegNames::R11 => { String::from("R11/T3") }
			RegNames::R12 => { String::from("R12/T4") }
			RegNames::R13 => { String::from("R13/T5") }
			RegNames::R14 => { String::from("R14/T6") }
			RegNames::R15 => { String::from("R15/T7") }

			RegNames::R16 => { String::from("R16/S0") }
			RegNames::R17 => { String::from("R17/S1") }
			RegNames::R18 => { String::from("R18/S2") }
			RegNames::R19 => { String::from("R19/S3") }
			RegNames::R20 => { String::from("R20/S4") }
			RegNames::R21 => { String::from("R21/S5") }
			RegNames::R22 => { String::from("R22/S6") }
			RegNames::R23 => { String::from("R23/S7") }

			RegNames::R24 => { String::from("R24/T8") }
			RegNames::R25 => { String::from("R25/T9") }

			RegNames::R26 => { String::from("R26/K0") }
			RegNames::R27 => { String::from("R27/K1") }
			RegNames::R28 => { String::from("R28/GP") }
			RegNames::R29 => { String::from("R29/SP") }
			RegNames::R30 => { String::from("R30/FP") }
			RegNames::R31 => { String::from("R31/RA") }

			RegNames::HI => { String::from("HI") }
			RegNames::LO => { String::from("LO") }
			RegNames::PC => { String::from("PC") }
			_ => { String::from("[NEED TO CONVERT REGISTER NAME TO R## FIRST]") }
		}
	}

	pub fn register_align(n: &RegNames) -> RegNames {
		let mut res = n.clone();
		match n {
			RegNames::ZERO => { res = RegNames::R0 }
			RegNames::At => { res = RegNames::R1 }
			RegNames::V0 => { res = RegNames::R2 }
			RegNames::V1 => { res = RegNames::R3 }
			RegNames::A0 => { res = RegNames::R4 }
			RegNames::A1 => { res = RegNames::R5 }
			RegNames::A2 => { res = RegNames::R6 }
			RegNames::A3 => { res = RegNames::R7 }

			RegNames::T0 => { res = RegNames::R8 }
			RegNames::T1 => { res = RegNames::R9 }
			RegNames::T2 => { res = RegNames::R10 }
			RegNames::T3 => { res = RegNames::R11 }
			RegNames::T4 => { res = RegNames::R12 }
			RegNames::T5 => { res = RegNames::R13 }
			RegNames::T6 => { res = RegNames::R14 }
			RegNames::T7 => { res = RegNames::R15 }
			RegNames::T8 => { res = RegNames::R24 } // The two out of place temp regs
			RegNames::T9 => { res = RegNames::R25 }

			RegNames::S0 => { res = RegNames::R16 }
			RegNames::S1 => { res = RegNames::R17 }
			RegNames::S2 => { res = RegNames::R18 }
			RegNames::S3 => { res = RegNames::R19 }
			RegNames::S4 => { res = RegNames::R20 }
			RegNames::S5 => { res = RegNames::R21 }
			RegNames::S6 => { res = RegNames::R22 }
			RegNames::S7 => { res = RegNames::R23 }

			RegNames::K0 => { res = RegNames::R26 }
			RegNames::K1 => { res = RegNames::R27 }
			RegNames::GP => { res = RegNames::R28 }
			RegNames::SP => { res = RegNames::R29 }
			RegNames::FP => { res = RegNames::R30 }
			RegNames::RA => { res = RegNames::R31 }
			_ => {}
		};
		res
	}

	pub fn idx_from_enum(n: &RegNames) -> usize {
		match n {
			RegNames::R0 => 0,
			RegNames::R1 => 1,
			RegNames::R2 => 2,
			RegNames::R3 => 3,
			RegNames::R4 => 4,
			RegNames::R5 => 5,
			RegNames::R6 => 6,
			RegNames::R7 => 7,
			RegNames::R8 => 8,
			RegNames::R9 => 9,
			RegNames::R10 => 10,
			RegNames::R11 => 11,
			RegNames::R12 => 12,
			RegNames::R13 => 13,
			RegNames::R14 => 14,
			RegNames::R15 => 15,
			RegNames::R16 => 16,
			RegNames::R17 => 17,
			RegNames::R18 => 18,
			RegNames::R19 => 19,
			RegNames::R20 => 20,
			RegNames::R21 => 21,
			RegNames::R22 => 22,
			RegNames::R23 => 23,
			RegNames::R24 => 24,
			RegNames::R25 => 25,
			RegNames::R26 => 26,
			RegNames::R27 => 27,
			RegNames::R28 => 28,
			RegNames::R29 => 29,
			RegNames::R30 => 30,
			RegNames::R31 => 31,

			RegNames::ZERO => 0,
			_ => 0
		}
	}
}

#[derive(Copy, Clone)]
pub struct Register {
	pub value: i32,
	pub name: RegNames,
}

impl Register {
	pub fn new(n: RegNames) -> Register {
		Register {
			value: 0,
			name: n,
		}
	}

	pub fn display_string(&self) -> String {
		format!("{}: {}", RegNames::to_str(&self.name), &self.value)
	}
}