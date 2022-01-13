# Author:	Colin Carlson
# Description:	Main file for peg jumping game

# syscall codes
PRINT_INT =     1
PRINT_STRING =  4
READ_STRING = 	8
PRINT_CHAR =    11
READ_INT =      5
EXIT =          10

#defines
NUM_ROWS	= 10				#this skips row/column math
NUM_COLUMNS	= 10				#board is still 7x7
						#max index = 64
GAME_BOARD_SIZE	= 100				#NUM_ROWS*NUM_COLUMNS
QUIT_INT	= -1


.data
.align 2 					#just to be safe
board_data:
	.space	GAME_BOARD_SIZE			#1 byte chars for now
.align 2
remaining_pegs:
	.word 	32 				#every board starts with 32 pegs
input_cache:
	.asciiz	"--" 				#stores the characters entered
welcome_stars:
	.asciiz	"   ************************\n" #print after message
welcome_message:
	.asciiz	"   **     Peg Puzzle     **\n"
number_row:
	.asciiz	"    0  1  2  3  4  5  6\n"
row_cap:
	.asciiz	"        +---------+\n"
row_end_start:
	.asciiz "       |"			#leading number is skipped
row_end_end:
	.asciiz "|      \n"
row_transition_start:
	.asciiz " +-----+"			#leading number is skipped
row_transition_end:
	.asciiz "+-----+\n"
row_edge_start:
	.asciiz " |"
row_edge_end:
	.asciiz "|\n"
new_line:
	.asciiz	"\n"

peg_to_move:
	.asciiz	"Enter the location of the peg to move (RC, -1 to quit): "
peg_new_location:
	.ascii	"Enter the location where the peg "	#80 column limit
	.asciiz	"is moving to (RC, -1 to quit): "	#is really annoying
player_quits:
	.asciiz	"Player quit.\n"
no_more_moves:
	.asciiz	"There are no more legal moves.\n"
#number of pegs left
pegs_left_message_start:
	.asciiz "You left "
#before printing this
pegs_left_message_end:
	.asciiz " pegs on the board.\n"

#if locat not on board
error_bad_location:
	.asciiz	"\nIllegal location.\n\n"
error_illegal_start:
	.asciiz	"\nIllegal move, no peg at source location.\n\n"
error_illegal_target:
	.asciiz	"\nIllegal move, destination location is occupied.\n\n"
error_mult_peg_jump:
	.asciiz	"\nIllegal move, can only jump over one peg, re-enter move.\n\n"
error_no_peg_jump:
	.asciiz	"\nIllegal move, no peg being jumped over, re-enter move.\n\n"
error_not_a_number:
	.asciiz "\nIllegal input, not a valid number.\n\n"

debug_string_up:
	.asciiz "U - Instruction Point Reached.\n"
debug_string_down:
	.asciiz "D - Instruction Point Reached.\n"
debug_string_left:
	.asciiz "L - Instruction Point Reached.\n"
debug_string_right:
	.asciiz "R - Instruction Point Reached.\n"

.text
.align 2	#required, code must be aligned


# 
# Name:		Main
# Description:	Builds a game board, loops over input, and parses the input
#		Input is a 2-digit number with the first digit being
#		the row number, second being the column
# Arguments:	None
# Returns:	None
# 
main:
	addi	$sp, $sp, -28		#move 6*4
	sw	$ra, 24($sp)
	sw	$s7, 20($sp)		#pegs remaining
	sw	$s6, 16($sp)		#
	sw	$s3, 12($sp)		#middle peg location
	sw	$s2, 8($sp)		#distance of start - end
	sw	$s1, 4($sp)		#end location
	sw	$s0, 0($sp) 		#start location

	jal	print_start_message
	#jal	print_board 		#for debug

	#
	#set up game data
	#

	jal	initialize_board
	li 	$s7, 32 			#start w/ 32 pegs

game_logic:
	li 	$t0, 1 				#if 1 peg left
	beq 	$s7, $t0, no_moves_left 	#then no moves left

	jal	print_board 			#print out the board
	#jal 	print_board_debug

    #viable moves check
	jal	check_board_status
	beq 	$v0, $zero, no_moves_left

game_input_sequence:
	jal	get_start_input
	addi	$s0, $v0, 0			#move start to s0
	addi	$t0, $zero, -1 			#if -1, quit
	beq	$t0, $v0, time_to_quit

	jal	get_target_input
	addi	$s1, $v0, 0			#move target to s0
	addi	$t0, $zero, -1 			#if -1, quit
	beq	$t0, $v0, time_to_quit

	#
	#check if distance is appropriate
	#
	#s2 = s0-s1
	sub 	$s2, $s0, $s1
	li 	$t0, 20
	beq 	$s2, $t0, valid_move_check
	li 	$t0, -20
	beq 	$s2, $t0, valid_move_check
	li 	$t0, 2
	beq 	$s2, $t0, valid_move_check
	li 	$t0, -2
	beq 	$s2, $t0, valid_move_check

	li	$v0, PRINT_STRING
	la	$a0, error_mult_peg_jump
	syscall
	j 	game_input_sequence
valid_move_check:
	
	#
	#check if jumping over peg
	#
	add 	$t0, $s0, $s1
	li 	$t1, 2
	div 	$t0, $t1 			#takes mean of s1,s2
	mflo 	$a0 				#t0//t1
	move 	$s3, $a0
	jal 	is_hole_occupied
	bne 	$v0, $zero, valid_jump_check

	li	$v0, PRINT_STRING
	la	$a0, error_no_peg_jump
	syscall
	j 	game_input_sequence
valid_jump_check:

	li 	$t1, 32 			#32 is space, 88 is X
	la	$s6, board_data
	add 	$t0, $s6, $s0 			#remove start peg
	sb 	$t1, 0($t0)

	add 	$t0, $s6, $s3
	sb 	$t1, 0($t0) 			#remove middle peg
	
	li 	$t1, 88
	add 	$t0, $s6, $s1
	sb 	$t1, 0($t0) 			#add end peg

	#adjust remaining pegs counter
	addi 	$s7, $s7, -1

	#cleanup
	li	$v0, PRINT_STRING
	la	$a0, new_line
	syscall

	j	game_logic
game_logic_end:

	lw	$ra, 24($sp)
	lw	$s7, 20($sp)
	lw	$s6, 16($sp)
	lw	$s3, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 28
	jr	$ra
time_to_quit:
	li	$v0, PRINT_STRING 		#End program functionality
	la	$a0, player_quits
	syscall

	j 	pegs_left
no_moves_left:
	li	$v0, PRINT_STRING
	la	$a0, no_more_moves
	syscall

	j 	pegs_left
pegs_left:
	li	$v0, PRINT_STRING
	la	$a0, pegs_left_message_start
	syscall

	li	$v0, PRINT_INT
	move 	$a0, $s7 			#prints out the remaining_pegs
	syscall

	li	$v0, PRINT_STRING
	la	$a0, pegs_left_message_end
	syscall

	li 	$v0, 10 			#quit syscall code
	syscall

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s3, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

	#
	# Program Terminates
	#

#
# Name:         print_start_message
# Description:  Prints the opening message
# Arguments:    None
# Returns:      None
#
print_start_message:
	addi	$sp, $sp, -12
	sw	$ra, 8($sp)		#safety first
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	li	$v0, PRINT_STRING       #prints the stars
	la	$a0, welcome_stars
	syscall

	li	$v0, PRINT_STRING       #prints the words in between
	la	$a0, welcome_message
	syscall

	li	$v0, PRINT_STRING       #prints the stars
	la	$a0, welcome_stars
	syscall
	
	li	$v0, PRINT_STRING	#new line
	la	$a0, new_line
	syscall
	
	lw	$ra, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 12
	jr	$ra

#
# Name:         initialize_board
# Description:  Sets up the board for a fresh game.
#		32 (space) is free, 88 (X) for a peg there
# Arguments:    None
# Returns:      None
#
initialize_board:
	addi	$sp, $sp, -24		#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)		#loop cap
	sw	$s3, 12($sp)		#extra
	sw	$s2, 8($sp)		#value being inserted to the board
	sw	$s1, 4($sp)		#loop counter
	sw	$s0, 0($sp)		#functions as iterator on board

	la	$s0, board_data
	addi	$s1, $zero, 0
	addi	$s7, $zero, GAME_BOARD_SIZE
	addi	$s2, $zero, 32

wipe_board_start:
	sb	$s2, 0($s0)		#store 32 @ board locat[x]
	beq	$s1, $s7, wipe_board_end
	addi	$s0, $s0, 1		#move the iterator +1 char
	addi	$s1, $s1, 1 		#counter++
	j 	wipe_board_start
wipe_board_end:
	
	#
	# Fills the playable game board with 'X's
	#

	la	$s0, board_data		#remember to reset on each part
	addi	$s2, $zero, 88		#32 is space, 88 is X
	sb	$s2, 2($s0)		#02
	sb	$s2, 3($s0)		#03
	sb	$s2, 4($s0)		#04

	sb	$s2, 12($s0)		#12
	sb	$s2, 13($s0)		#13
	sb	$s2, 14($s0)		#14

	sb	$s2, 20($s0)		#20
	sb	$s2, 21($s0)		#21
	sb	$s2, 22($s0)		#22
	sb	$s2, 23($s0)		#23
	sb	$s2, 24($s0)		#24
	sb	$s2, 25($s0)		#25
	sb	$s2, 26($s0)		#26

	sb	$s2, 30($s0)		#30
	sb	$s2, 31($s0)		#31
	sb	$s2, 32($s0)		#32
	sb	$s2, 34($s0)		#34, skip 33 middle spot
	sb	$s2, 35($s0)		#35
	sb	$s2, 36($s0)		#36

	sb	$s2, 40($s0)		#40
	sb	$s2, 41($s0)		#41
	sb	$s2, 42($s0)		#42
	sb	$s2, 43($s0)		#43
	sb	$s2, 44($s0)		#44
	sb	$s2, 45($s0)		#45
	sb	$s2, 46($s0)		#46

	sb	$s2, 52($s0)		#52
	sb	$s2, 53($s0)		#53
	sb	$s2, 54($s0)		#54

	sb	$s2, 62($s0)		#62
	sb	$s2, 63($s0)		#63
	sb	$s2, 64($s0)		#64

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s3, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         print_board
# Description:  prints the current board's data
# Arguments:    None
# Returns:      None
#
print_board:
	addi	$sp, $sp, -24		#move 6*4
	sw	$ra, 20($sp)
	sw	$v0, 16($sp)
	sw	$a0, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	la	$s0, board_data

	li 	$v0, PRINT_STRING	#v0: type of syscall
	la 	$a0, number_row 	#a0: location of string
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_cap
	syscall

	#row 0 -- indecies: 2 - 4

	li 	$v0, PRINT_INT
	li 	$a0, 0
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_end_start
	syscall

	addi 	$a0, $s0, 2 		#for indecies 02-04
	addi 	$a1, $zero, 3
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_end_end
	syscall

	#row 1

	li 	$v0, PRINT_INT
	li 	$a0, 1
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_transition_start
	syscall

	addi 	$a0, $s0, 12 		#for indecies 12-14
	addi 	$a1, $zero, 3
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_transition_end
	syscall

	#row 2

	li 	$v0, PRINT_INT
	li 	$a0, 2
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_start
	syscall

	addi 	$a0, $s0, 20 		#for indecies 20-26
	addi 	$a1, $zero, 7
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_end
	syscall

	#row 3

	li 	$v0, PRINT_INT
	li 	$a0, 3
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_start
	syscall

	addi 	$a0, $s0, 30 		#for indecies 30-36
	addi 	$a1, $zero, 7
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_end
	syscall

	#row 4

	li 	$v0, PRINT_INT
	li 	$a0, 4
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_start
	syscall

	addi 	$a0, $s0, 40 		#for indecies 40-46
	addi 	$a1, $zero, 7
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_edge_end
	syscall

	#row 5

	li 	$v0, PRINT_INT
	li 	$a0, 5
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_transition_start
	syscall

	addi 	$a0, $s0, 52 		#for indecies 52-54
	addi 	$a1, $zero, 3
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_transition_end
	syscall

	#row 6

	li 	$v0, PRINT_INT
	li 	$a0, 6
	syscall

	li 	$v0, PRINT_STRING
	la 	$a0, row_end_start
	syscall

	addi 	$a0, $s0, 62 		#for indecies 62-64
	addi 	$a1, $zero, 3
	jal	print_row_length

	li 	$v0, PRINT_STRING
	la 	$a0, row_end_end
	syscall

	# end 

	li 	$v0, PRINT_STRING
	la 	$a0, row_cap
	syscall



	#cleanup
	li	$v0, PRINT_STRING
	la	$a0, new_line
	syscall

	lw	$ra, 20($sp)
	lw	$v0, 16($sp)
	lw	$a0, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra


#
# Name:         print_board_debug
# Description:  prints the game board without formatting
# Arguments:    None
# Returns:      None
#
print_board_debug:
	addi	$sp, $sp, -24		#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	li 	$t0, 0
	li 	$s1, GAME_BOARD_SIZE 	#max board size for here
	la	$s0, board_data
print_board_debug_loop:
	beq 	$t0, $s1, print_board_debug_end

	add 	$a0, $s0, $t0
	addi 	$a1, $zero, 10
	jal	print_row_length
	li 	$v0, PRINT_STRING
	la 	$a0, new_line
	syscall

	addi 	$t0, $t0, 10
	j 	print_board_debug_loop
print_board_debug_end:

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         print_row_length
# Description:  Prints a row's segment
# Arguments:    $a0: address to start from, increments each iteration
#		$a1: the number of iterations
# Returns:      None
#
print_row_length:
	addi	$sp, $sp, -24		#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	addi	$s0, $a0, 0 		#address
	addi	$s1, $a1, 0 		#number of iterations

print_row_length_loop:
	beq	$s1, $zero, print_row_length_end

	li 	$v0, PRINT_CHAR
	li 	$a0, 32			#print double space
	syscall

	li 	$v0, PRINT_CHAR
	lb 	$a0, 0($s0)
	syscall

	li 	$v0, PRINT_CHAR
	li 	$a0, 32			#print double space
	syscall

	addi	$s0, $s0, 1
	addi	$s1, $s1, -1
	j 	print_row_length_loop
print_row_length_end:

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         get_start_input
# Description:  Gets the input for which peg to move.
# Arguments:    None
# Returns:      v0 = peg ID, before converting to row/column
#
get_start_input:
	addi	$sp, $sp, -24			#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

get_start_input_loop:
	li 	$v0, PRINT_STRING
	la 	$a0, peg_to_move 		#print start input string
	syscall

	li 	$v0, READ_INT 			#returns the int in $v0
	syscall
	add 	$s0, $zero, $v0 		#create a backup of the int

	addi 	$t0, $zero, -1
	bne 	$v0, $t0, is_start_quit_check
	j 	time_to_quit			#if int == -1, have to quit
is_start_quit_check:

	addi 	$a0, $s0, 0 			#set a0 for next method
	jal 	locat_on_board 			#1 for true, 0 false
	bne 	$zero, $v0, input_is_on_board
	li 	$v0, PRINT_STRING
	la 	$a0, error_bad_location 	#print bad location string
	syscall
	j 	get_start_input_loop 		#return to start
input_is_on_board:

	addi 	$a0, $s0, 0 			#setup a0 for peg check
	jal 	is_hole_occupied		#v0 = 1 for has peg, 0 is no
	bne 	$zero, $v0, peg_is_there 	#check that there is no peg
	li 	$v0, PRINT_STRING
	la 	$a0, error_illegal_start 	#if no peg, can't move stuff
	syscall
	j 	get_start_input_loop 		#error and restart
peg_is_there:

	move 	$v0, $s0

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         get_target_input
# Description:  Gets the input for where that peg should move
# Arguments:    None
# Returns:      v0 = peg ID, before converting to row/column
#
get_target_input:
	addi	$sp, $sp, -24			#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

get_target_input_loop:
	li 	$v0, PRINT_STRING
	la 	$a0, peg_new_location 		#print start input string
	syscall

	li 	$v0, READ_INT 			#returns the int in $v0
	syscall
	add 	$s0, $zero, $v0 		#create a backup of the int

	addi 	$t0, $zero, -1
	bne 	$v0, $t0, is_target_quit_check
	j 	time_to_quit			#if int == -1, have to quit
is_target_quit_check:

	addi 	$a0, $s0, 0 			#set a0 for next method
	jal 	locat_on_board 			#1 for true, 0 false
	bne 	$zero, $v0, target_is_on_board
	li 	$v0, PRINT_STRING
	la 	$a0, error_bad_location 	#print bad location string
	syscall
	j 	get_target_input_loop 		#return to start
target_is_on_board:

	addi 	$a0, $s0, 0 			#setup a0 for peg check
	jal 	is_hole_occupied		#v0 = 1 for has peg, 0 is no
	beq 	$zero, $v0, peg_is_there_target #check that there is a peg
	li 	$v0, PRINT_STRING
	la 	$a0, error_illegal_target 	#if no peg, can't move stuff
	syscall
	j 	get_target_input_loop 		#error and restart
peg_is_there_target:

	move 	$v0, $s0

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         locat_on_board
# Description:  
# Arguments:    a0 = the number to check
# Returns:      v0 = 1 for true, 0 for false
#
locat_on_board:
	addi	$sp, $sp, -24		#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	addi	$v0, $zero, 1 		#set to true, 'if's overwrite

					#t7 = trash
					#t6 = trash
	addi	$t2, $zero, 10 		#t2 = 10
	addi	$t3, $zero, 6 		#t3 = 6
	div 	$a0, $t2
	mflo	$t0			#lo(row) = $a0//10
	mfhi	$t1			#hi(col) = $a0 % 10

	#check max row/col bounds

	slt 	$t7, $t0, $zero 		#if row < 0: false
	beq	$t7, $zero, locat_neg_row_check	
	addi	$v0, $zero, 0
	j 	on_board_check_end
locat_neg_row_check:
	slt 	$t7, $t0, $zero 		#if col < 0: false
	beq	$t7, $zero, locat_neg_col_check	
	addi	$v0, $zero, 0
	j 	on_board_check_end
locat_neg_col_check:
	slt 	$t7, $t3, $t0 			#if 6 < row: false
	beq	$t7, $zero, locat_row_check	
	addi	$v0, $zero, 0
	j 	on_board_check_end
locat_row_check:
	slt 	$t7, $t3, $t1 			#if 6 < col: false
	beq	$t7, $zero, locat_col_check
	addi	$v0, $zero, 0
	j 	on_board_check_end
locat_col_check:

	#check quadrants

	addi	$t6, $zero, 2
	slt	$t7, $t1, $t6			#if col < 2: 1
	beq	$t7, $zero, left_col_check
	slt	$t7, $t0, $t6			#if row < 2: 1
	beq	$t7, $zero, left_col_check_mid
	addi	$v0, $zero, 0 			#v0 = 0
	j 	on_board_check_end
left_col_check_mid:
	addi	$t6, $zero, 4
	slt	$t7, $t6, $t0			#if 4 < row: 1
	beq	$t7, $zero, left_col_check
	addi	$v0, $zero, 0 			#v0 = 0
	j 	on_board_check_end
left_col_check:

	addi	$t6, $zero, 4
	slt	$t7, $t6, $t1			#if 4 < col: 1
	beq	$t7, $zero, right_col_check
	addi	$t6, $zero, 2
	slt	$t7, $t0, $t6			#if row < 2: 1
	beq	$t7, $zero, right_col_check_mid
	addi	$v0, $zero, 0 			#v0 = 0
	j 	on_board_check_end
right_col_check_mid:
	addi	$t6, $zero, 4
	slt	$t7, $t6, $t0			#if 4 < row: 1
	beq	$t7, $zero, right_col_check
	addi	$v0, $zero, 0 			#v0 = 0
	j 	on_board_check_end
right_col_check:

on_board_check_end:

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         is_hole_occupied
# Description:  
# Arguments:    a0 = the number to check
# Returns:      v0 = 1 for true, 0 for false
#
is_hole_occupied:
	addi	$sp, $sp, -24			#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s6, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	addi	$v0, $zero, 0
	addi 	$t7, $zero, 32 			#32 is space, 88 is X
	la 	$t0, board_data

	add 	$t0, $t0, $a0 			#board_data + a0
	lb 	$t1, 0($t0)
	beq 	$t1, $t7, occupy_check_end 	#beq if board[mem + a0] = 32
	addi 	$v0, $zero, 1
occupy_check_end:

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s6, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         check_board_status
# Description:  Checks each node on the board for potential moves
# Arguments: 	None
# Returns:      v0 = 1 for true, 0 for false
#
check_board_status:
	addi	$sp, $sp, -24				#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s3, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	li 	$s0, 0 					#moves counter
	li 	$s1, 0 					#index position
check_board_status_loop:
	bne 	$s0, $zero, check_board_status_end 	#end when >0 moves
	li 	$t0, GAME_BOARD_SIZE
	beq 	$s1, $t0, check_board_status_end 	#end: counter = b size
	
	addi 	$a0, $s1, 0 				#set up locat_on_board
	jal 	locat_on_board
	beq 	$v0, $zero, check_board_status_next 	#skip if not on board

	addi 	$a0, $s1, 0
	jal 	is_hole_occupied
	beq 	$v0, $zero, check_board_status_next 	#if empty, skip

	addi 	$a0, $s1, 0
	jal 	check_point_status
	beq 	$v0, $zero, check_board_status_next 	#skip counter check

	add 	$s0, $s0, $v0 				#add found moves

check_board_status_next:
	addi 	$s1, $s1, 1
	j 	check_board_status_loop
check_board_status_end:

	move 	$v0, $s0

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s3, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra

#
# Name:         check_point_status
# Description:  Checks each node on the board for potential moves
# Arguments: 	a0 = the point
# Returns:      v0 = 1 for true, 0 for false
#
check_point_status:
	addi	$sp, $sp, -24			#move 6*4
	sw	$ra, 20($sp)
	sw	$s7, 16($sp)
	sw	$s3, 12($sp)
	sw	$s2, 8($sp)
	sw	$s1, 4($sp)
	sw	$s0, 0($sp)

	addi 	$s0, $a0, 0 			#backup a0 to s0
	addi 	$s1, $zero, 0 			#set moves counter to 0


	#Upwards Check

	addi 	$a0, $s0, -20 			#current location up 2 rows
	jal 	locat_on_board
	beq 	$v0, $zero, up_direction_check 	#if off board, skip
	
	addi 	$a0, $s0, -20
	jal 	is_hole_occupied
	bne 	$v0, $zero, up_direction_check 	#if full, skip

	addi 	$a0, $s0, -10
	jal 	locat_on_board
	beq 	$v0, $zero, up_direction_check 	#if off board, skip

	addi 	$a0, $s0, -10
	jal 	is_hole_occupied 		#checks middle peg
	beq 	$v0, $zero, up_direction_check 	#if empty, skip

	addi 	$s1, $s1, 1 			#valid move! Increment!
up_direction_check:


	#Downwards Check

	addi 	$a0, $s0, 20 			#current location down 2 rows
	jal 	locat_on_board
	beq 	$v0, $0, down_direction_check 	#if off board, skip
	
	addi 	$a0, $s0, 20
	jal 	is_hole_occupied
	bne 	$v0, $0, down_direction_check 	#if full, skip

	addi 	$a0, $s0, 10
	jal 	locat_on_board
	beq 	$v0, $zero, down_direction_check #if off board, skip

	addi 	$a0, $s0, 10
	jal 	is_hole_occupied 		#checks middle peg
	beq 	$v0, $0, down_direction_check 	#if empty, skip

	addi 	$s1, $s1, 1 			#valid move! Increment!
down_direction_check:


	#Leftward Check

	addi 	$a0, $s0, -2 			#current location left 2
	jal 	locat_on_board
	beq 	$v0, $0, left_direction_check 	#if off board, skip
	
	addi 	$a0, $s0, -2
	jal 	is_hole_occupied
	bne 	$v0, $0, left_direction_check 	#if full, skip

	addi 	$a0, $s0, -1
	jal 	locat_on_board
	beq 	$v0, $zero, left_direction_check #if off board, skip

	addi 	$a0, $s0, -1
	jal 	is_hole_occupied 		#checks middle peg
	beq 	$v0, $0, left_direction_check 	#if empty, skip

	addi 	$s1, $s1, 1 			#valid move! Increment!
left_direction_check:


	#Rightward Check

	addi 	$a0, $s0, 2 			#current location right 2
	jal 	locat_on_board
	beq 	$v0, $0, right_direction_check 	#if off board, skip
	
	addi 	$a0, $s0, 2
	jal 	is_hole_occupied
	bne 	$v0, $0, right_direction_check 	#if full, skip

	addi 	$a0, $s0, 1
	jal 	locat_on_board
	beq 	$v0, $zero, right_direction_check #if off board, skip

	addi 	$a0, $s0, 1
	jal 	is_hole_occupied 		#checks middle peg
	beq 	$v0, $0, right_direction_check 	#if empty, skip

	addi 	$s1, $s1, 1 			#valid move! Increment!
right_direction_check:


	move 	$v0, $s1

	lw	$ra, 20($sp)
	lw	$s7, 16($sp)
	lw	$s3, 12($sp)
	lw	$s2, 8($sp)
	lw	$s1, 4($sp)
	lw	$s0, 0($sp)
	addi	$sp, $sp, 24
	jr	$ra
	
