.section .text
@ B test
MOV R0, #20
start:
ADD R1, R1, #420
SUBS R0, R0, #1
BNE start
ADD R1, R1, #69
@ BL test
MOV R2, #420
MOV R2, #424
BL handler
MOV R2, #428
MOV R2, #432
MOV R2, #436
B endhandler
handler:
MOV R1, #69
MOV PC, LR
endhandler:
MOV R1, #99
