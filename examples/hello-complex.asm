.orig x3000

HELLO .stringz "Hello, World!\n"
COUNTER .fill 5

LEA R0, HELLO
LD R1, COUNTER
PUTS
ADD R1, R1, #-1
BRp #-3
HALT

.end