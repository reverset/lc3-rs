.orig x3000
START
            add r0, r1, #15
            and r0, r1, r2
            not r2, r2          ; cool
            brnzp START

            brnzp OTHER
            jsr OTHER
OTHER

            brnzp START
            brnzp OTHER

            ld r0, VALUE
            ldr r0, r0, #0
            lea r0, VALUE

            jsr OTHER

            st r0, VALUE

VALUE       .fill #5
.end