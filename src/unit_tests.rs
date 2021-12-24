#[cfg(test)]
mod tests {
	use crate::mips_int;

	#[test]
	fn test_space10() {
		let data =
			".data\n\
			board:\n\
			.space 10"; // 10 BYTES, so 2.5 lines, round up to 3 lines of 0.
		let mut intr = mips_int::MipsInterpreter::new();
		intr.load_program(data);
		let code = intr.get_program_contents();
		let mut count = 0;
		for line in code.lines() {
			assert_eq!(line, "0");
			count += 1;
		}
		assert_eq!(count, 3);
	}

	#[test]
	fn test_space12() {
		let data =
			".data\n\
			board:\n\
			.space 12"; // 12 BYTES, so 3 lines
		let mut intr = mips_int::MipsInterpreter::new();
		intr.load_program(data);
		let code = intr.get_program_contents();
		let mut count = 0;
		for line in code.lines() {
			assert_eq!(line, "0");
			count += 1;
		}
		assert_eq!(count, 3);
	}
}
