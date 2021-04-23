.section .text
MOV R0, #69 @ Basic LDR/LDRB - STR/STRB test
STR R0, lmao
LDR R1, lmao
MOV R0, R1
MOV R2, #100
STR R0, [R2]
LDR R1, [R2]
MOV R0, R1
MOV R3, #1
STR R0, [R2, R3, LSL #2]!
LDR R1, [R2]
MOV R0, R1
LDR R0, endlmao @ self modifying code test
STR R0, test
STR R0, lmao
test: B endlmao
lmao: NOP
endlmao: MOV R0, #69
STRB R0, lmao
LDRB R1, lmao
MOV R0, R1
