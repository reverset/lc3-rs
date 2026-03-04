.orig x0000
.fill x0210
.end

.orig x0210
    LEA R0, HELLO
    PUTS
    RTI
    
HELLO .stringz "Hi!\n"
    
.end

.orig x3000
    TRAP x0000
    TRAP x0000
    TRAP x0000
    
    HALT
    
.end