# LC-3-_rs_

## Features
| Feature         | Info                                |
|-----------------|-------------------------------------|
| Virtual Machine | [Mostly implemented](#instructions) |
| Assembler       | soon ™️                             |
| Disassembler    | soon ™️                             |
| C Compiler      | soon ™️                             |

## References
- https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf (best source)

- https://en.wikipedia.org/wiki/Little_Computer_3 (A little inaccurate)
- https://www.cs.utexas.edu/~fussell/courses/cs310h/lectures/Lecture_10-310h.pdf
- https://acg.cis.upenn.edu/milom/cse240-Fall05/handouts/Ch09-a.pdf



## Instructions
| Instruction | Implemented |
|:-----------:|:-----------:|
|     ADD     |      ✅      |
|     AND     |      ✅      |
|     BR      |      ✅      |
|     JMP     |      ✅      |
|     JSR     |      ✅      |
|    JSRR     |      ✅      |
|     LD      |      ✅      |
|     LDI     |      ✅      |
|     LDR     |      ✅      |
|     LEA     |      ✅      |
|     NOT     |      ✅      |
|     RET     |      ✅      |
|     RTI     |      ❌      |
|     ST      |      ✅      |
|     STI     |      ✅      |
|     STR     |      ✅      |
|    TRAP     |      ✅      |
|  *reserved  |      ❌      |

* reserved causes an exception when used, which is not implemented yet

# Missing features
- Interrupts & privileged flag
- Device register assignments, namely:
  - 0xFE00 Keyboard status
  - 0xFE02 Keyboard data
