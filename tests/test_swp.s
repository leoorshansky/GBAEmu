.section .text
MOV R0, #300
MOV R4, #1020
SWPB R1, R4, [R0]
SWP R1, R4, [R0]
MOV R2, R1
