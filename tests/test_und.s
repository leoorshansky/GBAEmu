.section .text
B start
B und_handler
start:
MOV R0, #69
.word 3858759696
B endhandler
und_handler:
MOV R8, #69
MOVS PC, LR
endhandler:
MOV R9, #69
