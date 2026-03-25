.orig x3000
START
            add r0, r1, #15
            and r0, r1, r2
            not r2, r2          ; cool
            brnzp START
.end