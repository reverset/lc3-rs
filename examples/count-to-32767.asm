.orig x3000

        AND R0, R0, #0
        LD R1, GOAL
        
LOOP    ADD R0, R0, #1
        ADD R2, R0, R1
        BRn LOOP
        
        HALT
        
GOAL    .FILL x7FFF     ; 32,767

.end