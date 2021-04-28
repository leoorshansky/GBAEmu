.section startup
B resetHandler
NOP
NOP
NOP
NOP
NOP
resetHandler:
MOV SP, #0x20000
BL main
MOV R8, R0
MOV R8, R1
MOV R8, R2
MOV R8, R3
B end
