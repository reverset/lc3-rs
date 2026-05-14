.orig x3000
            brnzp START
            

RESULT      .blkw #1
TOS         .blkw #10
RPN         .fill #0

PLUS        .fill x2B
MINUS       .fill x2D
TIMES       .fill x2A
DIV         .fill x2F
PRINT       .fill x2E


ASCII_0     .fill x30
ASCII_9     .fill x39

START
            lea r0, TITLE           ; print title and instructions
            puts
            
            ld r0, NL
            out
            
            lea r0, INSTRUCTION
            puts
            
            ld r0, NL
            out
            
            ld r1, ASCII_0          ; negate ASCII_0, we use r1 from now on as this
            not r1, r1
            add r1, r1, #1
            st r1, ASCII_0
            
            ld r2, PLUS             ; negate PLUS
            not r2, r2
            add r2, r2, #1
            st r2, PLUS
            
            ld r2, MINUS            ; negate MINUS
            not r2, r2
            add r2, r2, #1
            st r2, MINUS
            
            ld r2, TIMES            ; negate PLUS
            not r2, r2
            add r2, r2, #1
            st r2, TIMES
            
            ld r2, DIV              ; negate DIV
            not r2, r2
            add r2, r2, #1
            st r2, DIV
            
            ld r2, PRINT            ; negate PRINT
            not r2, r2
            add r2, r2, #1
            st r2, PRINT
            
            
            lea r4, RPN             ; rpn stack pointer
            
MAIN_LOOP
            GETC                    ; read char
            
            ld r2, PLUS
            add r2, r0, r2
            brz PLUS_OP
            
            ld r2, MINUS
            add r2, r0, r2
            brz MINUS_OP
            
            ld r2, TIMES
            add r2, r0, r2
            brz TIMES_OP
            
            ld r2, PRINT
            add r2, r0, r2
            brz PRINT_OP
            
            ld r2, DIV
            add r2, r0, r2
            brz DIV_OP
            
            add r2, r0, r1          ; ascii -> num
            add r3, r2, #-10
            brn IS_NUM             ; number!
            
            
            ; if we are here, no valid character was typed
            brnzp MAIN_LOOP
            
IS_NUM  
            out
            and r0, r0, #0
            add r0, r2, #0          ; move r2 -> r0
            jsr PUSH
            ;add r4, r4, #-1         ; push num onto stack
            ;str r2, r4, #0
        
            brnzp MAIN_LOOP         ; repeat

PLUS_OP
            out
            jsr POP
            
            add r2, r0, #0          ; move r0 -> r2
            
            jsr POP
            
            add r0, r0, r2          ; do addition

            jsr PUSH
            
            brnzp MAIN_LOOP
            
MINUS_OP
            out
            jsr POP                 ; pop newest
            and r2, r2, #0
            add r2, r0, #0          ; newest -> r2
            
            jsr POP                 ; pop oldest
            
            not r2, r2
            add r2, r2, #1          ; negate newest
            
            add r0, r2, r0          ; oldest - newest
            
            jsr PUSH
            
            brnzp MAIN_LOOP
            
TIMES_OP
            out
            
            and r3, r3, #0          ; result
            
            jsr POP
            add r2, r0, #0          ; r0 -> r2
            brz TIMES_DONE
            
            jsr POP
TIMES_LOOP
            add r3, r3, r0          ; r3+r0 -> r3
            add r2, r2, #-1         ; r2-1 -> r2 (dec counter)
            brp TIMES_LOOP          ; repeat while counter > 0

TIMES_DONE
            add r0, r3, #0          ; r3 -> r0
            jsr PUSH
            brnzp MAIN_LOOP

DIV_OP
            out
            jsr POP                 ; pop newest
            add r2, r0, #0          ; newest -> r2
            brz DIV_ZERO            ; check if zero
            
            jsr POP                 ; pop oldest
            and r3, r3, #0          ; initialize result
            
            not r2, r2
            add r2, r2, #1          ; negate r2 (newest)
            
            ; oldest / newest
DIV_LOOP
            add r0, r0, r2          ; r0-r2 -> r0
            brn DIV_DONE
            
            add r3, r3, #1          ; otherwise inc r3
            
            brnzp DIV_LOOP
            
DIV_DONE
            add r0, r3, #0          ; r3 -> r0
            
            jsr PUSH
            
            brnzp MAIN_LOOP

DIV_ZERO
            ld r0, NL
            out
            lea r0, DIV_ZERO_MSG
            puts
            halt
            

DIV_ZERO_MSG    .stringz "Cannot divide by zero!"

PRINT_OP
            out
            jsr POP
            st r0, RESULT
            halt

PUSH
            st r3, PUSH_POP_TEMP        ; save r3
            
            lea r3, TOS
            not r3, r3
            add r3, r3, #1
            add r3, r4, r3              ; check if we are below the stack
            brnz NO_STACK_LEFT_2
            
            add r4, r4, #-1             ; actually push otherwise
            str r0, r4, #0
            
            ld r3, PUSH_POP_TEMP        ; restore r3
            
            ret

POP
            st r3, PUSH_POP_TEMP        ; save r3
            
            lea r3, RPN
            not r3, r3
            add r3, r3, #1
            add r3, r4, r3              ; check if we are below the stack
            brzp NO_STACK_LEFT
            
            ldr r0, r4, #0              ; pop otherwise
            add r4, r4, #1
            
            ld r3, PUSH_POP_TEMP        ; restore r3
            
            ret

PUSH_POP_TEMP    .blkw 1                ; temporary storage for r3.

NO_STACK_LEFT
            ld r0, NL
            out
            lea r0, NOT_ENOUGH_NUMBERS
            puts
            ld r0, NL
            out
            
            halt
            
NO_STACK_LEFT_2
            ld r0, NL
            out
            lea r0, TOO_MANY_NUMBERS
            puts
            ld r0, NL
            out
            
            halt
            
NL          .fill x0A
TITLE       .stringz "Single Digit RPN Calculator"
INSTRUCTION .stringz "Enter 0-9 or +, -, *, /, ., to place TOS in RESULT and end."

NOT_ENOUGH_NUMBERS  .stringz "Please enter a valid amount of numbers for the operator."
TOO_MANY_NUMBERS    .stringz "Too many numbers entered! Stack overflow!"

.end