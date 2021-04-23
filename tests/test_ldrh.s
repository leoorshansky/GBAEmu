.section .text
MOV R0, #69 @ Basic LDR(S)H - STRH test
STRH R0, lmao
LDRH R1, lmao
MOV R0, R1
MOV R2, #100
STRH R0, [R2]
LDRH R1, [R2]
MOV R0, R1
SUB R0, R0, #70
STRH R0, [R2]
LDRSH R1, [R2]
MOV R0, R1
MOV R3, #2
STRH R0, [R2, R3]!
LDRH R1, [R2]
MOV R0, R1
LDRH R0, endlmao @ self modifying code test
STRH R0, test
STRH R0, lmao
LDRH R0, endlmao + 2
STRH R0, test + 2
STRH R0, lmao + 2
NOP
NOP
test: B endlmao
lmao: NOP
endlmao: MOV R0, #69
STRB R0, lmao
LDRSB R1, lmao
MOV R0, R1
