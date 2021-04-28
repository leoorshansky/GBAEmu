.section .text
B start
B start
B swi_handler
start:
MOV R0, #69
SWI #69
B endhandler
swi_handler:
MOV R8, #69
MOVS PC, LR
endhandler:
MOV R9, #69