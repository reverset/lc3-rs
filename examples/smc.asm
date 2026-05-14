.ORIG x3000
; SMC RPN Calculator

; -- Initialize System --
START       LEA R0, HEADING    ; Load welcome header
            PUTS               ; Print to console
            BRnzp RESET
HEADING     .STRINGZ "SMC RPN calculator\nEnter 0-9 or +, -, *, /, or\n. to display result on TOS\n"

RESET       LD R6, R_ST_BASE   ; Reset stack ptr
            LD R5, R_ST_PTR    ; Get address of STACK_PTR
            STR R6, R5, #0     ; Save stack ptr
            BRnzp MAIN_PROMPT
R_ST_BASE   .FILL x4000
R_ST_PTR    .FILL STACK_PTR

; -- Main Prompt Loop --
MAIN_PROMPT LEA R0, PROMPT_RDY ; Load "> "
            PUTS               ; Print prompt
            BRnzp MP_CONT
PROMPT_RDY  .STRINGZ "> "

MP_CONT     AND R0, R0, #0     ; Clear R0
            ST R0, VAL         ; Clear input value
            ST R0, IS_NEG      ; Clear neg flag
            ST R0, NUM_DIGITS  ; Clear digit count
            ST R0, IS_OP       ; Clear op flag
            ST R0, OP_CHAR     ; Clear op char

; -- Input Parsing --
READ_LOOP   LD R1, SEED        ; Load random seed
            ADD R1, R1, #1     ; Increment seed
            ST R1, SEED        ; Save new seed
            GETC               ; Read user input
            OUT                ; Echo to screen
            
            ADD R1, R0, #-10   ; Check if LF
            BRz PROC_ENTER     ; Handle enter key
            ADD R1, R0, #-13   ; Check if CR
            BRz PROC_ENTER     ; Handle enter key
            
            LD R1, CHAR_LWR_Q  ; Check 'q'
            ADD R1, R0, R1     
            BRnp SKIP_GG1
            LD R1, PTR_GG
            JMP R1
SKIP_GG1    LD R1, CHAR_UPR_Q  ; Check 'Q'
            ADD R1, R0, R1     
            BRnp SKIP_GG2
            LD R1, PTR_GG
            JMP R1
SKIP_GG2    BRnzp SKIP_GG_PTR
PTR_GG      .FILL GUESS_GAME
SKIP_GG_PTR
            
            LD R1, CHAR_MINUS  ; Check '-'
            ADD R1, R0, R1     
            BRnp NOT_MINUS     ; Skip if not '-'
            LD R2, NUM_DIGITS  ; Check parsed digits
            BRp MINUS_OP       ; Has digits, is operator
            LD R2, IS_OP       ; Check parsed op
            BRp MINUS_OP       ; Has op, is operator
            LD R2, IS_NEG      ; Check neg flag
            BRp MINUS_OP       ; Has neg, is operator
            ADD R2, R2, #1     ; Set neg flag
            ST R2, IS_NEG      ; Save neg flag
            BRnzp READ_LOOP    ; Read next char
MINUS_OP    ST R0, OP_CHAR     ; Save '-' as op
            AND R2, R2, #0     ; Clear R2
            ADD R2, R2, #1     ; Set op flag = 1
            ST R2, IS_OP       ; Save op flag
            BRnzp READ_LOOP    ; Read next char

NOT_MINUS   LD R1, CHAR_PLUS   ; Check '+'
            ADD R1, R0, R1     
            BRnp NOT_PLUS      ; Skip if not '+'
            ST R0, OP_CHAR     ; Save '+' as op
            AND R2, R2, #0     ; Clear R2
            ADD R2, R2, #1     ; Set op flag = 1
            ST R2, IS_OP       ; Save op flag
            BRnzp READ_LOOP    ; Read next char
            
NOT_PLUS    LD R1, CHAR_MULT   ; Check '*'
            ADD R1, R0, R1     
            BRnp NOT_MULT      ; Skip if not '*'
            ST R0, OP_CHAR     ; Save '*' as op
            AND R2, R2, #0     ; Clear R2
            ADD R2, R2, #1     ; Set op flag = 1
            ST R2, IS_OP       ; Save op flag
            BRnzp READ_LOOP    ; Read next char

NOT_MULT    LD R1, CHAR_DIV    ; Check '/'
            ADD R1, R0, R1     
            BRnp NOT_DIV       ; Skip if not '/'
            ST R0, OP_CHAR     ; Save '/' as op
            AND R2, R2, #0     ; Clear R2
            ADD R2, R2, #1     ; Set op flag = 1
            ST R2, IS_OP       ; Save op flag
            BRnzp READ_LOOP    ; Read next char

NOT_DIV     LD R1, CHAR_DOT    ; Check '.'
            ADD R1, R0, R1     
            BRnp NOT_DOT       ; Skip if not '.'
            ST R0, OP_CHAR     ; Save '.' as op
            AND R2, R2, #0     ; Clear R2
            ADD R2, R2, #1     ; Set op flag = 1
            ST R2, IS_OP       ; Save op flag
            BRnzp READ_LOOP    ; Read next char

NOT_DOT     LD R1, CHAR_0      ; Check digit min
            ADD R1, R0, R1     
            BRn READ_LOOP      ; Ignore < '0'
            LD R1, NCHAR_9     ; Check digit max
            ADD R1, R0, R1     
            BRp READ_LOOP      ; Ignore > '9'
            
            LD R2, NUM_DIGITS  ; Load digit count
            ADD R2, R2, #1     ; Increment count
            ST R2, NUM_DIGITS  ; Save count
            
            LD R2, VAL         ; Load parsed val
            ADD R3, R2, R2     ; val * 2
            ADD R4, R3, R3     ; val * 4
            ADD R4, R4, R4     ; val * 8
            ADD R2, R4, R3     ; val * 10
            
            LD R1, CHAR_0      ; Char offset
            ADD R0, R0, R1     ; Convert ascii -> int
            ADD R2, R2, R0     ; Add digit
            ST R2, VAL         ; Save new val
            BRnzp READ_LOOP    ; Read next char

; -- Handle Enter Key --
PROC_ENTER  LD R1, NUM_DIGITS  ; Check if we have num
            BRz PE_NO_NUM      ; No number parsed
            LD R2, IS_OP       ; Check if we have op
            BRp ERR_MATH_J     ; Both op/num is bad
            
            LD R0, VAL         ; Load parsed val
            LD R2, IS_NEG      ; Check neg flag
            BRz PE_PUSH        ; Pos number, push
            NOT R0, R0         ; 2's complement
            ADD R0, R0, #1     
PE_PUSH     JSR PUSH_VAL       ; Push to stack
            BRnzp JUMP_MP_L1   ; Wait for input
JUMP_MP_L1  LD R1, P_MP_L1
            JMP R1
P_MP_L1     .FILL MAIN_PROMPT

PE_NO_NUM   LD R2, IS_OP       ; Check if op parsed
            BRp PE_DO_OP       ; Execute operation
            LD R2, IS_NEG      ; Check lone minus
            BRp PE_LONE_M      ; Handle as subtract
            BRnzp JUMP_MP_L1   ; Empty enter, retry

PE_LONE_M   LD R0, PCHAR_M     ; Load '-'
            ST R0, OP_CHAR     ; Set as operation
            BRnzp PE_DO_OP     ; Execute subtract
ERR_MATH_J  LD R1, P_ERR_MATH
            JMP R1
P_ERR_MATH  .FILL ERR_MATH

; -- Execute Operations --
PE_DO_OP    LD R0, OP_CHAR     ; Load operator char
            LD R1, NCHAR_DOT   ; Check for '.'
            ADD R1, R0, R1     
            BRz DO_PRINT       ; Print top stack
            
            JSR POP_VAL        ; Pop right operand
            ADD R2, R0, #0     ; R2 = right op
            JSR POP_VAL        ; Pop left operand
            ADD R1, R0, #0     ; R1 = left op
            LD R0, OP_CHAR     ; Reload op char
            
            LD R3, NCHAR_PLUS  ; Check '+'
            ADD R3, R0, R3     
            BRz DO_ADD         ; Do addition
            LD R3, NCHAR_M     ; Check '-'
            ADD R3, R0, R3     
            BRz DO_SUB         ; Do subtraction
            LD R3, NCHAR_MULT  ; Check '*'
            ADD R3, R0, R3     
            BRz DO_MULT        ; Do multiply
            BRnzp DO_DIV       ; Do divide


; ==========================================
; GLOBALS AND CONSTANTS FOR PARSER
; ==========================================
            BRnzp DO_ADD

VAL         .FILL 0
IS_NEG      .FILL 0
NUM_DIGITS  .FILL 0
IS_OP       .FILL 0
OP_CHAR     .FILL 0
SEED        .FILL 0

CHAR_0      .FILL xFFD0
NCHAR_9     .FILL xFFC7
CHAR_MINUS  .FILL xFFD3
PCHAR_M     .FILL x002D
CHAR_PLUS   .FILL xFFD5
CHAR_MULT   .FILL xFFD6
CHAR_DIV    .FILL xFFD1
CHAR_DOT    .FILL xFFD2
NCHAR_PLUS  .FILL xFFD5
NCHAR_M     .FILL xFFD3
NCHAR_MULT  .FILL xFFD6
NCHAR_DOT   .FILL xFFD2
CHAR_LWR_Q  .FILL xFF8F
CHAR_UPR_Q  .FILL xFFAF
; ==========================================


; -- Math Operations --
DO_ADD      ADD R0, R1, R2     ; R0 = R1 + R2
            ADD R1, R1, #0     ; Check R1 sign
            BRn ADD_R1_NEG     
            ADD R2, R2, #0     ; Check R2 sign
            BRn ADD_OK         ; Diff signs ok
            ADD R0, R0, #0     ; Check R0 sign
            BRn ERR_OFLOW_J    ; Pos+Pos=Neg -> OF
            BRnzp ADD_DONE     
ADD_R1_NEG  ADD R2, R2, #0     ; Check R2 sign
            BRzp ADD_OK        ; Diff signs ok
            ADD R0, R0, #0     ; Check R0 sign
            BRzp ERR_OFLOW_J   ; Neg+Neg=Pos -> OF
ADD_OK      
ADD_DONE    JSR PUSH_VAL       ; Push result
            BRnzp JUMP_MP_L2

DO_SUB      NOT R2, R2         ; 2's complement
            ADD R2, R2, #1     ; R2 = -R2
            BRnzp DO_ADD       ; R1 + (-R2)

DO_MULT     AND R0, R0, #0     ; Init product
            AND R4, R4, #0     ; Init sign flag
            ADD R1, R1, #0     ; Check R1 sign
            BRzp M_R1_POS      
            NOT R1, R1         ; R1 = |R1|
            ADD R1, R1, #1     
            NOT R4, R4         ; Toggle sign flag
M_R1_POS    ADD R2, R2, #0     ; Check R2 sign
            BRzp M_R2_POS      
            NOT R2, R2         ; R2 = |R2|
            ADD R2, R2, #1     
            NOT R4, R4         ; Toggle sign flag
M_R2_POS    
M_LOOP      ADD R2, R2, #0     ; Check multiplier
            BRz M_DONE         ; Done when 0
            ADD R0, R0, R1     ; Add multiplicand
            BRn ERR_OFLOW_J    ; Unsigned OF
            ADD R2, R2, #-1    ; Decrement
            BRnzp M_LOOP       
M_DONE      ADD R4, R4, #0     ; Check sign flag
            BRz M_PUSH         ; Pos result
            NOT R0, R0         ; Apply negative
            ADD R0, R0, #1     
M_PUSH      JSR PUSH_VAL       ; Push product
            BRnzp JUMP_MP_L2

ERR_OFLOW_J LD R1, P_ERR_OFLOW
            JMP R1
P_ERR_OFLOW .FILL ERR_OFLOW

DO_DIV      ADD R2, R2, #0     ; Check divisor
            BRz ERR_MATH_J2    ; Div by zero
            AND R0, R0, #0     ; Init quotient
            AND R4, R4, #0     ; Init sign flag
            ADD R1, R1, #0     ; Check R1 sign
            BRzp D_R1_POS      
            NOT R1, R1         ; R1 = |R1|
            ADD R1, R1, #1     
            NOT R4, R4         ; Toggle sign flag
D_R1_POS    ADD R2, R2, #0     ; Check R2 sign
            BRzp D_R2_POS      
            NOT R2, R2         ; R2 = |R2|
            ADD R2, R2, #1     
            NOT R4, R4         ; Toggle sign flag
D_R2_POS    NOT R5, R2         
            ADD R5, R5, #1     ; R5 = -R2
D_LOOP      ADD R1, R1, R5     ; R1 = R1 - R2
            BRn D_DONE         ; Stop if neg
            ADD R0, R0, #1     ; Inc quotient
            BRnzp D_LOOP       
D_DONE      ADD R4, R4, #0     ; Check sign flag
            BRz D_PUSH         ; Pos result
            NOT R0, R0         ; Apply negative
            ADD R0, R0, #1     
D_PUSH      JSR PUSH_VAL       ; Push quotient
            BRnzp JUMP_MP_L2

ERR_MATH_J2 LD R1, P_ERR_MATH2
            JMP R1
P_ERR_MATH2 .FILL ERR_MATH

JUMP_MP_L2  LD R1, P_MP_L2
            JMP R1
P_MP_L2     .FILL MAIN_PROMPT

DO_PRINT    LD R6, STACK_PTR   ; Check stack depth
            LD R5, STACK_BASE  
            NOT R5, R5         
            ADD R5, R5, #1     
            ADD R5, R6, R5     
            BRz ERR_STACK_J    ; Empty stack error
            LDR R1, R6, #0     ; Peek TOS
            
            LEA R0, TOS_STR    
            PUTS               ; Print "TOS: "
            BRnzp DP_CONT
TOS_STR     .STRINGZ "TOS: "
DP_CONT     JSR PRINT_NUM      ; Print value
            LD R0, DP_CHAR_LF  
            OUT                ; Print newline
            BRnzp JUMP_MP_L2   
DP_CHAR_LF  .FILL x000A

ERR_STACK_J LD R1, P_ERR_STACK
            JMP R1
P_ERR_STACK .FILL ERR_STACK

; -- Error Handlers --
ERR_STACK   LEA R0, ESTR_STK   ; Load stack err
            PUTS               ; Print '$'
            BRnzp JUMP_RST     ; Reset calculator
ESTR_STK    .STRINGZ "$\n"

ERR_MATH    LEA R0, ESTR_MTH   ; Load math err
            PUTS               ; Print '?'
            BRnzp JUMP_RST     ; Reset calculator
ESTR_MTH    .STRINGZ "?\n"

ERR_OFLOW   LEA R0, ESTR_OVF   ; Load overflow err
            PUTS               ; Print '!'
            BRnzp JUMP_RST     ; Reset calculator
ESTR_OVF    .STRINGZ "!\n"

JUMP_RST    LD R1, PTR_RST
            JMP R1
PTR_RST     .FILL RESET

; -- Stack Operations --
PUSH_VAL    LD R6, STACK_PTR   ; Load stack ptr
            LD R5, STACK_LIM   ; Load limit ptr
            NOT R5, R5         
            ADD R5, R5, #1     
            ADD R5, R6, R5     
            BRnz ERR_STACK_J   ; Stack overflow
            ADD R6, R6, #-1    ; Move SP down
            STR R0, R6, #0     ; Store value
            ST R6, STACK_PTR   ; Save SP
            RET                

STACK_BASE  .FILL x4000
STACK_PTR   .FILL x4000
STACK_LIM   .FILL x3FF0

POP_VAL     LD R6, STACK_PTR   ; Load stack ptr
            LD R5, STACK_BASE  ; Load base ptr
            NOT R5, R5         
            ADD R5, R5, #1     
            ADD R5, R6, R5     
            BRz ERR_STACK_J    ; Stack underflow
            LDR R0, R6, #0     ; Read value
            ADD R6, R6, #1     ; Move SP up
            ST R6, STACK_PTR   ; Save SP
            RET                

; -- Print Decimal Number --
PRINT_NUM   ST R7, PN_R7       ; Save R7
            ADD R1, R1, #0     ; Check value
            BRnp PN_CHK_NEG    
            LD R0, PN_PCHAR_0  ; Zero case
            OUT                ; Print '0'
            BRnzp PN_END       
PN_CHK_NEG  BRp PN_POS         
            LD R0, PN_PCHAR_M  ; Print minus
            OUT                
            NOT R1, R1         ; Make positive
            ADD R1, R1, #1     
PN_POS      AND R6, R6, #0     ; Printed flag = 0
            LEA R2, DIVISORS   ; Load div table
PN_DIG_LOOP LDR R3, R2, #0     ; Load divisor
            BRz PN_LAST_DIG    ; If 0, do ones
            AND R4, R4, #0     ; Digit count = 0
PN_SUB_LOOP ADD R1, R1, R3     ; Subtract div
            BRn PN_OVSUB       ; Over subtracted
            ADD R4, R4, #1     ; Count++
            BRnzp PN_SUB_LOOP  
PN_OVSUB    NOT R5, R3         ; Restore value
            ADD R5, R5, #1     
            ADD R1, R1, R5     
            ADD R4, R4, #0     ; Check count
            BRp PN_DO_PRT      ; Count > 0, print
            ADD R6, R6, #0     ; Check printed flag
            BRz PN_NXT_DIV     ; Skip leading zero
PN_DO_PRT   LD R0, PN_PCHAR_0  
            ADD R0, R0, R4     
            OUT                ; Print digit
            ADD R6, R6, #1     ; Set printed flag
PN_NXT_DIV  ADD R2, R2, #1     ; Next divisor
            BRnzp PN_DIG_LOOP  
PN_LAST_DIG LD R0, PN_PCHAR_0  
            ADD R0, R0, R1     
            OUT                ; Print ones digit
PN_END      LD R7, PN_R7       ; Restore R7
            RET                

PN_PCHAR_0  .FILL x0030
PN_PCHAR_M  .FILL x002D
PN_R7       .FILL 0
DIVISORS    .FILL #-10000
            .FILL #-1000
            .FILL #-100
            .FILL #-10
            .FILL #0

; -- Easter Egg: Guessing Game --
GUESS_GAME  LD R1, GG_P_SEED
            LDR R0, R1, #0     ; Load RNG seed
            ADD R0, R0, #7     ; Scramble seed
            STR R0, R1, #0     ; Save new seed
GG_MOD      ADD R0, R0, #-10   ; Modulo 10
            BRp GG_MOD         
            BRz GG_MOD_Z       
            ADD R0, R0, #10    
GG_MOD_Z    ST R0, GG_TGT      ; Save target val
            
            LEA R0, GG_PRMT    
            PUTS               ; Print game prompt
            BRnzp GG_R_TRIES
GG_PRMT     .STRINGZ "\nI'm sorry Dave, I'm afraid I can't do that.\n\n--- GUESSING GAME ---\nGuess a number 0-9 (3 tries):\n"
GG_R_TRIES  AND R4, R4, #0     ; Reset tries
GG_LOOP     GETC               ; Read guess
            OUT                ; Echo
            ADD R1, R0, #-10   ; Check LF
            BRz GG_LOOP        
            ADD R1, R0, #-13   ; Check CR
            BRz GG_LOOP        
            LD R1, GG_CHAR_0   
            ADD R2, R0, R1     ; Convert digit
            BRn GG_LOOP        ; Ignore < '0'
            ADD R3, R2, #-9    
            BRp GG_LOOP        ; Ignore > '9'
            ADD R4, R4, #1     ; tries++
            LD R0, GG_CHAR_LF  
            OUT                ; Print newline
            LD R1, GG_TGT      ; Load target
            NOT R1, R1         
            ADD R1, R1, #1     
            ADD R3, R2, R1     ; cmp guess, target
            BRz GG_WIN         ; Equal -> win!
            
            ADD R5, R4, #-3    ; Check if tries == 3
            BRz GG_LOSE        ; Out of tries -> lose
            
            ADD R3, R2, R1     ; Reload diff
            BRp GG_HIGH        ; Too high
            LEA R0, GG_LOW     
            PUTS               ; Print low
            BRnzp GG_LOOP      
GG_LOW      .STRINGZ "Too low!\n"

GG_HIGH     LEA R0, GG_HI      
            PUTS               ; Print high
            BRnzp GG_LOOP      
GG_HI       .STRINGZ "Too high!\n"

GG_LOSE     LEA R0, GG_LSTR    
            PUTS               ; Print lose msg
            BRnzp GG_JUMP_MP   
GG_LSTR     .STRINGZ "better luck next time chump\n"

GG_WIN      LEA R0, GG_WSTR    
            PUTS               ; Print win msg
            BRnzp GG_JUMP_MP   
GG_WSTR     .STRINGZ "Congratulations, here are your 58008\n"

GG_JUMP_MP  LD R1, GG_P_MP
            JMP R1

GG_TGT      .FILL 0
GG_P_SEED   .FILL SEED
GG_CHAR_0   .FILL xFFD0
GG_CHAR_LF  .FILL x000A
GG_P_MP     .FILL MAIN_PROMPT

.END
