.orig x3000

; comment

LEA R0, PROMPT
PUTS
GETC
OUT
ADD R1, R0, #0
LD R0, NEWLINE
OUT
ADD R0, R1, #0

LD R2, NUM_OFFSET
ADD R1, R0, R2

BRz END

LEA R0, HELLO ; another comment
LOOP PUTS
ADD R1, R1, #-1
BRp LOOP

END
HALT

HELLO       .stringz "Hello, World!\n"
PROMPT      .stringz "How many times (1 char please) (0..=9): "

NUM_OFFSET  .fill #-48
NEWLINE     .fill x0A

.end