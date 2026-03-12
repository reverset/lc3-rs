.ORIG x0000
.FILL x0200
.END

.ORIG x0200

LOOP    LDI R0, KBSR
        BRzp LOOP
        LDI R0, KBDR
        OUT
        RTI

KBSR    .FILL   xFE00
KBDR    .FILL   xFE02

.END

.ORIG x3000
        GETC
        OUT
        TRAP x0000
        HALT

.END