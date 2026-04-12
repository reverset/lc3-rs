.orig x0
.fill x0300
.end

.orig x0300
            rti
.end

.orig x3000

; hiii

.fill #30
            add r0, r1, r7
            add r0, r0, xA
            trap x0

            .stringz "Hello World\n"
            .fill x30
            .blkw 5
.end