.section .text
MOV R0, #0x0001
MOV R1, R0
MOV R2, R0
MOV R3, R0
MOV R4, R0
MOV R5, R0
@ Conditions test
ADDS R0, R0, R0, LSL #1
MOVNE R0, #8
MOVEQ R0, #420
SUBS R0, R0, #8
MOVNE R0, #420
MOVEQ R0, #8
ADDS R0, R0, #0
MOVCS R0, #420
MOVCC R0, #8
SUBS R0, R0, #1
MOVCS R0, #8
MOVCC R0, #420
SUBS R0, R0, #9
MOVMI R0, #8
MOVPL R0, #420
SUBS R0, R0, #3
MOVPL R0, #8
MOVMI R0, #420
@ Operand2 Test
ADDS R0, R0, #100
ADDS R0, R0, #101
ADDS R0, R0, #102
ADDS R0, R0, #103
ADDS R0, R0, #256
MOV R0, #8
MOV R0, R0, LSR #2
MOV R0, R0, ROR #2
MOV R0, R0, ASR #5
MOV R0, R0, RRX
MOV R0, R0, LSR #31
MOV R0, #32
MOV R0, R0, ROR #6
MOV R0, R0, ASR #31
MOV R1, #32
MOV R0, R0, LSL R1
MOV R0, R0, LSR R1
SUB R0, R0, #5
MOV R0, R0, ROR R1
MOV R0, R0, ASR R1
SUB R0, R0, #5
MOV R0, R0, LSR #1
MOV R0, R0, ASR R1
@ All ALU test
MOV R0, #10
MOV R1, #5
ANDS R2, R0, R1
MOVNE R2, #420
TST R0, R1
MOVNE R2, #420
ANDS R2, R0, R0
CMP R2, R0
MOVNE R2, #420
EOR R2, R0, R1
SUBS R2, R2, #0b1111
MOVNE R2, #420
RSB R2, R1, R0
SUBS R2, R2, R1
MOVNE R2, #420
SUBS R2, R0, #1
ADC R2, R2, #0
SUBS R2, R2, R0
MOVNE R2, #420
SUBS R2, R0, #11
ADC R2, R2, #0
SUBS R2, R2, R0
MOVEQ R2, #420
SUBS R2, R0, #1
SBCS R2, R1, R1
MOVNE R2, #420
SUBS R2, R0, #11
SBCS R2, R1, #4
MOVNE R2, #420
SUBS R2, R0, #1
RSCS R2, R1, R1
MOVNE R2, #420
SUBS R2, R0, #11
RSCS R2, R1, #6
MOVNE R2, #420
ORR R2, R0, R1
SUBS R2, R2, #0b1111
MOVNE R2, #420
BIC R2, R0, R1
SUBS R2, R2, R0
MOVNE R2, #420
MVN R2, R0
MVN R2, R2
SUBS R2, R2, R0
MOVNE R2, #420