// i386 System V layout, cross-checked against gcc 13.3.0 -m32 (elf32-i386) in the
// container from docker.mk. The host gcc does not target elf32-i386 and disagrees on
// long double and bit-field packing, so it cannot be used as the oracle here.

size!(abi_size_0, "struct S { double m0; unsigned short m1[1]; signed char m2; };", "struct S", 12);
bits!(
    abi_bits_0,
    "struct S { double m0; unsigned short m1[1]; signed char m2; };",
    "S",
    &[("m0", 0), ("m1", 64), ("m2", 80)]
);
size!(abi_size_1, "struct S { int m0 : 13; signed int m1 : 27; unsigned int m2; long double m3; };", "struct S", 24);
bits!(
    abi_bits_1,
    "struct S { int m0 : 13; signed int m1 : 27; unsigned int m2; long double m3; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96)]
);
size!(
    abi_size_2,
    "union I2_0 { signed char f0; char * f1; unsigned short f2; }; struct S { signed char m0; char * m1; int m2; int * m3; signed char m4; };",
    "struct S",
    20
);
bits!(
    abi_bits_2,
    "union I2_0 { signed char f0; char * f1; unsigned short f2; }; struct S { signed char m0; char * m1; int m2; int * m3; signed char m4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96), ("m4", 128)]
);
size!(abi_size_3, "union I3_0 { int f0; int * f1; char * f2; }; struct S { unsigned char m0; };", "struct S", 1);
bits!(abi_bits_3, "union I3_0 { int f0; int * f1; char * f2; }; struct S { unsigned char m0; };", "S", &[("m0", 0)]);
size!(abi_size_4, "struct S { unsigned int : 11; long m1[3]; double m2[3]; };", "struct S", 40);
bits!(abi_bits_4, "struct S { unsigned int : 11; long m1[3]; double m2[3]; };", "S", &[("m1", 32), ("m2", 128)]);
size!(
    abi_size_5,
    "union I5_0 { unsigned char f0; }; struct S { int : 16; double m1; char * m2; unsigned short m3; };",
    "struct S",
    20
);
bits!(
    abi_bits_5,
    "union I5_0 { unsigned char f0; }; struct S { int : 16; double m1; char * m2; unsigned short m3; };",
    "S",
    &[("m1", 32), ("m2", 96), ("m3", 128)]
);
size!(
    abi_size_6,
    "union I6_0 { double f0; }; struct S { short m0[1]; unsigned short m1; unsigned long m2; char * m3; };",
    "struct S",
    12
);
bits!(
    abi_bits_6,
    "union I6_0 { double f0; }; struct S { short m0[1]; unsigned short m1; unsigned long m2; char * m3; };",
    "S",
    &[("m0", 0), ("m1", 16), ("m2", 32), ("m3", 64)]
);
size!(abi_size_7, "union S { unsigned long m0; char m1; };", "union S", 4);
bits!(abi_bits_7, "union S { unsigned long m0; char m1; };", "S", &[("m0", 0), ("m1", 0)]);
size!(abi_size_8, "struct S { unsigned int m0; signed int : 13; char m2; char m3; char * m4; };", "struct S", 12);
bits!(
    abi_bits_8,
    "struct S { unsigned int m0; signed int : 13; char m2; char m3; char * m4; };",
    "S",
    &[("m0", 0), ("m2", 48), ("m3", 56), ("m4", 64)]
);
size!(
    abi_size_9,
    "union I9_0 { unsigned char f0; char * f1; signed char f2; }; struct S { float m0; int m1[2]; signed char m2; union I9_0 m3; };",
    "struct S",
    20
);
bits!(
    abi_bits_9,
    "union I9_0 { unsigned char f0; char * f1; signed char f2; }; struct S { float m0; int m1[2]; signed char m2; union I9_0 m3; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 96), ("m3", 128)]
);
size!(
    abi_size_10,
    "union I10_0 { void * f0; }; struct S { double m0; unsigned long m1; signed char m2; unsigned short m3; };",
    "struct S",
    16
);
bits!(
    abi_bits_10,
    "union I10_0 { void * f0; }; struct S { double m0; unsigned long m1; signed char m2; unsigned short m3; };",
    "S",
    &[("m0", 0), ("m1", 64), ("m2", 96), ("m3", 112)]
);
size!(abi_size_11, "struct S { float m0[1]; signed int m1 : 2; char * m2; };", "struct S", 12);
bits!(
    abi_bits_11,
    "struct S { float m0[1]; signed int m1 : 2; char * m2; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(abi_size_12, "union I12_0 { unsigned short f0; }; struct S { int : 0; int m9; };", "struct S", 4);
bits!(abi_bits_12, "union I12_0 { unsigned short f0; }; struct S { int : 0; int m9; };", "S", &[("m9", 0)]);
size!(abi_size_13, "union S { signed int : 12; int m9; };", "union S", 4);
bits!(abi_bits_13, "union S { signed int : 12; int m9; };", "S", &[("m9", 0)]);
size!(abi_size_14, "struct S { unsigned long m0[4]; };", "struct S", 16);
bits!(abi_bits_14, "struct S { unsigned long m0[4]; };", "S", &[("m0", 0)]);
size!(abi_size_15, "struct I15_0 { short f0; }; union S { struct I15_0 m0; void * m1; };", "union S", 4);
bits!(
    abi_bits_15,
    "struct I15_0 { short f0; }; union S { struct I15_0 m0; void * m1; };",
    "S",
    &[("m0", 0), ("m1", 0)]
);
size!(
    abi_size_16,
    "struct I16_0 { signed char f0; char f1; int f2; }; struct S { signed int m0 : 12; unsigned char m1; long double m2[3]; };",
    "struct S",
    40
);
bits!(
    abi_bits_16,
    "struct I16_0 { signed char f0; char f1; int f2; }; struct S { signed int m0 : 12; unsigned char m1; long double m2[3]; };",
    "S",
    &[("m0", 0), ("m1", 16), ("m2", 32)]
);
size!(abi_size_17, "struct I17_0 { unsigned long f0; float f1; }; struct S { double m0; };", "struct S", 8);
bits!(abi_bits_17, "struct I17_0 { unsigned long f0; float f1; }; struct S { double m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_18,
    "union I18_0 { short f0; float f1; }; struct S { int m0 : 15; signed int : 16; union I18_0 m2; };",
    "struct S",
    8
);
bits!(
    abi_bits_18,
    "union I18_0 { short f0; float f1; }; struct S { int m0 : 15; signed int : 16; union I18_0 m2; };",
    "S",
    &[("m0", 0), ("m2", 32)]
);
size!(abi_size_19, "struct S { unsigned char m0; double m1; long m2; int m3; int m4 : 10; };", "struct S", 24);
bits!(
    abi_bits_19,
    "struct S { unsigned char m0; double m1; long m2; int m3; int m4 : 10; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 96), ("m3", 128), ("m4", 160)]
);
size!(
    abi_size_20,
    "struct S { signed int m0 : 15; long m1; unsigned int : 4; void * m3; unsigned int m4[2]; };",
    "struct S",
    24
);
bits!(
    abi_bits_20,
    "struct S { signed int m0 : 15; long m1; unsigned int : 4; void * m3; unsigned int m4[2]; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m3", 96), ("m4", 128)]
);
size!(
    abi_size_21,
    "struct I21_0 { long double f0; float f1; }; union S { char m0; long m1[1]; unsigned int m2 : 6; long m3; char m4; long m5; };",
    "union S",
    4
);
bits!(
    abi_bits_21,
    "struct I21_0 { long double f0; float f1; }; union S { char m0; long m1[1]; unsigned int m2 : 6; long m3; char m4; long m5; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0), ("m4", 0), ("m5", 0)]
);
size!(abi_size_22, "struct I22_0 { short f0; float f1; }; struct S { int m0 : 5; short m1; };", "struct S", 4);
bits!(
    abi_bits_22,
    "struct I22_0 { short f0; float f1; }; struct S { int m0 : 5; short m1; };",
    "S",
    &[("m0", 0), ("m1", 16)]
);
size!(
    abi_size_23,
    "union I23_0 { unsigned long f0; }; union S { union I23_0 m0; unsigned short m1; int : 13; union I23_0 m3; int m4[4]; };",
    "union S",
    16
);
bits!(
    abi_bits_23,
    "union I23_0 { unsigned long f0; }; union S { union I23_0 m0; unsigned short m1; int : 13; union I23_0 m3; int m4[4]; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m3", 0), ("m4", 0)]
);
size!(abi_size_24, "union S { unsigned int : 3; char * m1; signed char m2[3]; };", "union S", 4);
bits!(abi_bits_24, "union S { unsigned int : 3; char * m1; signed char m2[3]; };", "S", &[("m1", 0), ("m2", 0)]);
size!(
    abi_size_25,
    "struct S { unsigned int m0[1]; unsigned int m1 : 11; signed char m2[1]; double m3[1]; };",
    "struct S",
    16
);
bits!(
    abi_bits_25,
    "struct S { unsigned int m0[1]; unsigned int m1 : 11; signed char m2[1]; double m3[1]; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 48), ("m3", 64)]
);
size!(
    abi_size_26,
    "struct S { unsigned short m0; int : 0; signed int m2 : 22; unsigned long m3[4]; };",
    "struct S",
    24
);
bits!(
    abi_bits_26,
    "struct S { unsigned short m0; int : 0; signed int m2 : 22; unsigned long m3[4]; };",
    "S",
    &[("m0", 0), ("m2", 32), ("m3", 64)]
);
size!(
    abi_size_27,
    "union S { long double m0[2]; unsigned int : 10; unsigned int : 10; signed int : 11; unsigned int m4 : 8; signed char m5; };",
    "union S",
    24
);
bits!(
    abi_bits_27,
    "union S { long double m0[2]; unsigned int : 10; unsigned int : 10; signed int : 11; unsigned int m4 : 8; signed char m5; };",
    "S",
    &[("m0", 0), ("m4", 0), ("m5", 0)]
);
size!(
    abi_size_28,
    "union I28_0 { void * f0; }; struct S { char m0; unsigned int m1 : 12; long double m2; };",
    "struct S",
    16
);
bits!(
    abi_bits_28,
    "union I28_0 { void * f0; }; struct S { char m0; unsigned int m1 : 12; long double m2; };",
    "S",
    &[("m0", 0), ("m1", 8), ("m2", 32)]
);
size!(abi_size_29, "union S { unsigned char m0[4]; signed int : 14; int m2 : 4; };", "union S", 4);
bits!(abi_bits_29, "union S { unsigned char m0[4]; signed int : 14; int m2 : 4; };", "S", &[("m0", 0), ("m2", 0)]);
size!(abi_size_30, "struct S { int m0; double m1[1]; int m2 : 5; char * m3; float m4; };", "struct S", 24);
bits!(
    abi_bits_30,
    "struct S { int m0; double m1[1]; int m2 : 5; char * m3; float m4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 96), ("m3", 128), ("m4", 160)]
);
size!(abi_size_31, "struct S { unsigned short m0[3]; };", "struct S", 6);
bits!(abi_bits_31, "struct S { unsigned short m0[3]; };", "S", &[("m0", 0)]);
size!(
    abi_size_32,
    "struct I32_0 { long f0; }; struct S { signed int m0 : 5; struct I32_0 m1; unsigned char m2; unsigned int : 4; float m4; };",
    "struct S",
    16
);
bits!(
    abi_bits_32,
    "struct I32_0 { long f0; }; struct S { signed int m0 : 5; struct I32_0 m1; unsigned char m2; unsigned int : 4; float m4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m4", 96)]
);
size!(abi_size_33, "struct S { unsigned short m0[2]; };", "struct S", 4);
bits!(abi_bits_33, "struct S { unsigned short m0[2]; };", "S", &[("m0", 0)]);
size!(
    abi_size_34,
    "struct I34_0 { long double f0; int f1; }; struct S { double m0; void * m1[1]; long m2; signed int m3 : 26; double m4; };",
    "struct S",
    28
);
bits!(
    abi_bits_34,
    "struct I34_0 { long double f0; int f1; }; struct S { double m0; void * m1[1]; long m2; signed int m3 : 26; double m4; };",
    "S",
    &[("m0", 0), ("m1", 64), ("m2", 96), ("m3", 128), ("m4", 160)]
);
size!(
    abi_size_35,
    "struct S { signed int : 6; signed char m1; int m2 : 27; int m3; unsigned char m4; int m5 : 6; };",
    "struct S",
    16
);
bits!(
    abi_bits_35,
    "struct S { signed int : 6; signed char m1; int m2 : 27; int m3; unsigned char m4; int m5 : 6; };",
    "S",
    &[("m1", 8), ("m2", 32), ("m3", 64), ("m4", 96), ("m5", 104)]
);
size!(
    abi_size_36,
    "struct I36_0 { float f0; double f1; unsigned long f2; }; struct S { float m0; signed int : 7; void * m2[1]; };",
    "struct S",
    12
);
bits!(
    abi_bits_36,
    "struct I36_0 { float f0; double f1; unsigned long f2; }; struct S { float m0; signed int : 7; void * m2[1]; };",
    "S",
    &[("m0", 0), ("m2", 64)]
);
size!(abi_size_37, "struct S { unsigned int m0 : 17; };", "struct S", 4);
bits!(abi_bits_37, "struct S { unsigned int m0 : 17; };", "S", &[("m0", 0)]);
size!(
    abi_size_38,
    "struct I38_0 { void * f0; }; struct S { char m0; struct I38_0 m1; signed int m2 : 24; };",
    "struct S",
    12
);
bits!(
    abi_bits_38,
    "struct I38_0 { void * f0; }; struct S { char m0; struct I38_0 m1; signed int m2 : 24; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(
    abi_size_39,
    "struct I39_0 { double f0; unsigned char f1; }; struct S { unsigned int : 6; struct I39_0 m1; unsigned short m2; unsigned int m3; short m4[2]; };",
    "struct S",
    28
);
bits!(
    abi_bits_39,
    "struct I39_0 { double f0; unsigned char f1; }; struct S { unsigned int : 6; struct I39_0 m1; unsigned short m2; unsigned int m3; short m4[2]; };",
    "S",
    &[("m1", 32), ("m2", 128), ("m3", 160), ("m4", 192)]
);
size!(
    abi_size_40,
    "union I40_0 { int * f0; signed char f1; }; struct S { int : 0; char * m1[3]; float m2; };",
    "struct S",
    16
);
bits!(
    abi_bits_40,
    "union I40_0 { int * f0; signed char f1; }; struct S { int : 0; char * m1[3]; float m2; };",
    "S",
    &[("m1", 0), ("m2", 96)]
);
size!(abi_size_41, "struct I41_0 { void * f0; }; struct S { int : 6; int m9; };", "struct S", 8);
bits!(abi_bits_41, "struct I41_0 { void * f0; }; struct S { int : 6; int m9; };", "S", &[("m9", 32)]);
size!(abi_size_42, "struct S { unsigned char m0; };", "struct S", 1);
bits!(abi_bits_42, "struct S { unsigned char m0; };", "S", &[("m0", 0)]);
size!(abi_size_43, "union S { double m0; };", "union S", 8);
bits!(abi_bits_43, "union S { double m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_44,
    "union I44_0 { char f0; }; struct S { signed int m0 : 16; union I44_0 m1; unsigned int m2 : 26; int : 6; signed int m4 : 24; };",
    "struct S",
    12
);
bits!(
    abi_bits_44,
    "union I44_0 { char f0; }; struct S { signed int m0 : 16; union I44_0 m1; unsigned int m2 : 26; int : 6; signed int m4 : 24; };",
    "S",
    &[("m0", 0), ("m1", 16), ("m2", 32), ("m4", 64)]
);
size!(
    abi_size_45,
    "struct I45_0 { unsigned int f0; }; union S { unsigned int m0[3]; signed int m1 : 12; double m2; unsigned short m3; };",
    "union S",
    12
);
bits!(
    abi_bits_45,
    "struct I45_0 { unsigned int f0; }; union S { unsigned int m0[3]; signed int m1 : 12; double m2; unsigned short m3; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(abi_size_46, "struct S { int m0[2]; long double m1[2]; unsigned short m2; };", "struct S", 36);
bits!(
    abi_bits_46,
    "struct S { int m0[2]; long double m1[2]; unsigned short m2; };",
    "S",
    &[("m0", 0), ("m1", 64), ("m2", 256)]
);
size!(
    abi_size_47,
    "struct I47_0 { short f0; float f1; }; struct S { signed int : 11; void * m1; float m2; };",
    "struct S",
    12
);
bits!(
    abi_bits_47,
    "struct I47_0 { short f0; float f1; }; struct S { signed int : 11; void * m1; float m2; };",
    "S",
    &[("m1", 32), ("m2", 64)]
);
size!(
    abi_size_48,
    "struct S { signed int m0 : 21; int : 0; signed int : 3; char m3; signed char m4[1]; int : 0; };",
    "struct S",
    8
);
bits!(
    abi_bits_48,
    "struct S { signed int m0 : 21; int : 0; signed int : 3; char m3; signed char m4[1]; int : 0; };",
    "S",
    &[("m0", 0), ("m3", 40), ("m4", 48)]
);
size!(
    abi_size_49,
    "struct I49_0 { void * f0; double f1; float f2; }; union S { long m0; int : 0; float m2; short m3[2]; int : 0; };",
    "union S",
    4
);
bits!(
    abi_bits_49,
    "struct I49_0 { void * f0; double f1; float f2; }; union S { long m0; int : 0; float m2; short m3[2]; int : 0; };",
    "S",
    &[("m0", 0), ("m2", 0), ("m3", 0)]
);
size!(
    abi_size_50,
    "union I50_0 { unsigned char f0; char * f1; unsigned long f2; }; union S { unsigned int : 2; unsigned int m1 : 32; double m2[2]; long double m3; void * m4; };",
    "union S",
    16
);
bits!(
    abi_bits_50,
    "union I50_0 { unsigned char f0; char * f1; unsigned long f2; }; union S { unsigned int : 2; unsigned int m1 : 32; double m2[2]; long double m3; void * m4; };",
    "S",
    &[("m1", 0), ("m2", 0), ("m3", 0), ("m4", 0)]
);
size!(
    abi_size_51,
    "union I51_0 { long f0; unsigned char f1; signed char f2; }; union S { char * m0; int m1; unsigned char m2; unsigned int m3 : 19; };",
    "union S",
    4
);
bits!(
    abi_bits_51,
    "union I51_0 { long f0; unsigned char f1; signed char f2; }; union S { char * m0; int m1; unsigned char m2; unsigned int m3 : 19; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(abi_size_52, "union I52_0 { int f0; int * f1; }; struct S { double m0; };", "struct S", 8);
bits!(abi_bits_52, "union I52_0 { int f0; int * f1; }; struct S { double m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_53,
    "struct I53_0 { unsigned char f0; char * f1; unsigned int f2; }; struct S { unsigned int m0 : 29; signed char m1; unsigned int m2 : 27; int : 0; char * m4; struct I53_0 m5; };",
    "struct S",
    28
);
bits!(
    abi_bits_53,
    "struct I53_0 { unsigned char f0; char * f1; unsigned int f2; }; struct S { unsigned int m0 : 29; signed char m1; unsigned int m2 : 27; int : 0; char * m4; struct I53_0 m5; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m4", 96), ("m5", 128)]
);
size!(
    abi_size_54,
    "union I54_0 { char * f0; unsigned long f1; }; struct S { int m0 : 21; int m1; long m2; long double m3; int m4[2]; };",
    "struct S",
    32
);
bits!(
    abi_bits_54,
    "union I54_0 { char * f0; unsigned long f1; }; struct S { int m0 : 21; int m1; long m2; long double m3; int m4[2]; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96), ("m4", 192)]
);
size!(
    abi_size_55,
    "struct I55_0 { int * f0; double f1; }; struct S { unsigned short m0; signed int m1 : 6; char * m2; short m3; };",
    "struct S",
    12
);
bits!(
    abi_bits_55,
    "struct I55_0 { int * f0; double f1; }; struct S { unsigned short m0; signed int m1 : 6; char * m2; short m3; };",
    "S",
    &[("m0", 0), ("m1", 16), ("m2", 32), ("m3", 64)]
);
size!(abi_size_56, "struct S { unsigned int : 16; int m9; };", "struct S", 8);
bits!(abi_bits_56, "struct S { unsigned int : 16; int m9; };", "S", &[("m9", 32)]);
size!(abi_size_57, "union S { long m0[3]; char m1; unsigned int m2 : 30; double m3; };", "union S", 12);
bits!(
    abi_bits_57,
    "union S { long m0[3]; char m1; unsigned int m2 : 30; double m3; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(
    abi_size_58,
    "struct S { unsigned int m0 : 22; unsigned int : 9; unsigned int : 3; unsigned int m3 : 31; unsigned int : 3; };",
    "struct S",
    16
);
bits!(
    abi_bits_58,
    "struct S { unsigned int m0 : 22; unsigned int : 9; unsigned int : 3; unsigned int m3 : 31; unsigned int : 3; };",
    "S",
    &[("m0", 0), ("m3", 64)]
);
size!(abi_size_59, "struct S { unsigned int m0 : 21; };", "struct S", 4);
bits!(abi_bits_59, "struct S { unsigned int m0 : 21; };", "S", &[("m0", 0)]);
size!(abi_size_60, "struct S { char * m0; signed int m1 : 13; long double m2; unsigned int : 7; };", "struct S", 24);
bits!(
    abi_bits_60,
    "struct S { char * m0; signed int m1 : 13; long double m2; unsigned int : 7; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(abi_size_61, "struct S { signed int : 13; int m1 : 17; int m2 : 20; };", "struct S", 8);
bits!(abi_bits_61, "struct S { signed int : 13; int m1 : 17; int m2 : 20; };", "S", &[("m1", 13), ("m2", 32)]);
size!(
    abi_size_62,
    "union I62_0 { long double f0; long double f1; }; struct S { void * m0; signed int m1 : 29; };",
    "struct S",
    8
);
bits!(
    abi_bits_62,
    "union I62_0 { long double f0; long double f1; }; struct S { void * m0; signed int m1 : 29; };",
    "S",
    &[("m0", 0), ("m1", 32)]
);
size!(abi_size_63, "union S { signed char m0; float m1; unsigned int m2 : 28; int m3; };", "union S", 4);
bits!(
    abi_bits_63,
    "union S { signed char m0; float m1; unsigned int m2 : 28; int m3; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(
    abi_size_64,
    "union I64_0 { char f0; unsigned long f1; char * f2; }; struct S { int m0 : 6; float m1; int m2; unsigned int m3 : 14; };",
    "struct S",
    16
);
bits!(
    abi_bits_64,
    "union I64_0 { char f0; unsigned long f1; char * f2; }; struct S { int m0 : 6; float m1; int m2; unsigned int m3 : 14; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96)]
);
size!(
    abi_size_65,
    "struct I65_0 { float f0; }; struct S { int m0 : 7; long double m1; float m2[3]; long double m3[4]; struct I65_0 m4; };",
    "struct S",
    80
);
bits!(
    abi_bits_65,
    "struct I65_0 { float f0; }; struct S { int m0 : 7; long double m1; float m2[3]; long double m3[4]; struct I65_0 m4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 128), ("m3", 224), ("m4", 608)]
);
size!(
    abi_size_66,
    "union I66_0 { int f0; signed char f1; double f2; }; struct S { short m0; long m1; int * m2; };",
    "struct S",
    12
);
bits!(
    abi_bits_66,
    "union I66_0 { int f0; signed char f1; double f2; }; struct S { short m0; long m1; int * m2; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(abi_size_67, "struct S { unsigned short m0; };", "struct S", 2);
bits!(abi_bits_67, "struct S { unsigned short m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_68,
    "struct I68_0 { unsigned long f0; int f1; long double f2; }; union S { char * m0[4]; long double m1[3]; unsigned int m2 : 25; struct I68_0 m3; };",
    "union S",
    36
);
bits!(
    abi_bits_68,
    "struct I68_0 { unsigned long f0; int f1; long double f2; }; union S { char * m0[4]; long double m1[3]; unsigned int m2 : 25; struct I68_0 m3; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(abi_size_69, "struct S { unsigned int m0 : 28; double m1[3]; int m2 : 18; };", "struct S", 32);
bits!(
    abi_bits_69,
    "struct S { unsigned int m0 : 28; double m1[3]; int m2 : 18; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 224)]
);
size!(
    abi_size_70,
    "union I70_0 { float f0; signed char f1; }; struct S { unsigned short m0[2]; unsigned int : 11; };",
    "struct S",
    6
);
bits!(
    abi_bits_70,
    "union I70_0 { float f0; signed char f1; }; struct S { unsigned short m0[2]; unsigned int : 11; };",
    "S",
    &[("m0", 0)]
);
size!(
    abi_size_71,
    "union I71_0 { unsigned long f0; long f1; int f2; }; struct S { int m0 : 18; unsigned int : 6; int m2[1]; unsigned int m3 : 14; };",
    "struct S",
    12
);
bits!(
    abi_bits_71,
    "union I71_0 { unsigned long f0; long f1; int f2; }; struct S { int m0 : 18; unsigned int : 6; int m2[1]; unsigned int m3 : 14; };",
    "S",
    &[("m0", 0), ("m2", 32), ("m3", 64)]
);
size!(abi_size_72, "struct S { long m0; float m1; unsigned int m2 : 18; };", "struct S", 12);
bits!(abi_bits_72, "struct S { long m0; float m1; unsigned int m2 : 18; };", "S", &[("m0", 0), ("m1", 32), ("m2", 64)]);
size!(
    abi_size_73,
    "union I73_0 { void * f0; float f1; }; struct S { union I73_0 m0; signed int m1 : 24; signed int m2 : 9; int : 15; };",
    "struct S",
    12
);
bits!(
    abi_bits_73,
    "union I73_0 { void * f0; float f1; }; struct S { union I73_0 m0; signed int m1 : 24; signed int m2 : 9; int : 15; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(abi_size_74, "struct S { int m0 : 3; char m1[2]; signed int : 7; };", "struct S", 4);
bits!(abi_bits_74, "struct S { int m0 : 3; char m1[2]; signed int : 7; };", "S", &[("m0", 0), ("m1", 8)]);
size!(
    abi_size_75,
    "union I75_0 { long double f0; }; struct S { union I75_0 m0; char m1[1]; union I75_0 m2; };",
    "struct S",
    28
);
bits!(
    abi_bits_75,
    "union I75_0 { long double f0; }; struct S { union I75_0 m0; char m1[1]; union I75_0 m2; };",
    "S",
    &[("m0", 0), ("m1", 96), ("m2", 128)]
);
size!(
    abi_size_76,
    "struct I76_0 { char * f0; long double f1; double f2; }; union S { signed int : 4; unsigned int m1[4]; };",
    "union S",
    16
);
bits!(
    abi_bits_76,
    "struct I76_0 { char * f0; long double f1; double f2; }; union S { signed int : 4; unsigned int m1[4]; };",
    "S",
    &[("m1", 0)]
);
size!(abi_size_77, "struct S { short m0; };", "struct S", 2);
bits!(abi_bits_77, "struct S { short m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_78,
    "struct I78_0 { unsigned int f0; short f1; void * f2; }; struct S { float m0; int * m1; unsigned int m2 : 25; };",
    "struct S",
    12
);
bits!(
    abi_bits_78,
    "struct I78_0 { unsigned int f0; short f1; void * f2; }; struct S { float m0; int * m1; unsigned int m2 : 25; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(
    abi_size_79,
    "union I79_0 { char f0; unsigned long f1; }; struct S { signed int m0 : 6; int : 10; };",
    "struct S",
    4
);
bits!(
    abi_bits_79,
    "union I79_0 { char f0; unsigned long f1; }; struct S { signed int m0 : 6; int : 10; };",
    "S",
    &[("m0", 0)]
);
size!(
    abi_size_80,
    "struct I80_0 { int * f0; void * f1; }; union S { long m0; char * m1; int m2 : 11; unsigned char m3[4]; unsigned long m4; };",
    "union S",
    4
);
bits!(
    abi_bits_80,
    "struct I80_0 { int * f0; void * f1; }; union S { long m0; char * m1; int m2 : 11; unsigned char m3[4]; unsigned long m4; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0), ("m4", 0)]
);
size!(abi_size_81, "struct S { long double m0[4]; char * m1[3]; int m2 : 4; };", "struct S", 64);
bits!(
    abi_bits_81,
    "struct S { long double m0[4]; char * m1[3]; int m2 : 4; };",
    "S",
    &[("m0", 0), ("m1", 384), ("m2", 480)]
);
size!(
    abi_size_82,
    "union I82_0 { float f0; unsigned char f1; unsigned long f2; }; struct S { unsigned char m0[2]; unsigned char m1; char * m2[1]; char m3[1]; signed int : 15; };",
    "struct S",
    12
);
bits!(
    abi_bits_82,
    "union I82_0 { float f0; unsigned char f1; unsigned long f2; }; struct S { unsigned char m0[2]; unsigned char m1; char * m2[1]; char m3[1]; signed int : 15; };",
    "S",
    &[("m0", 0), ("m1", 16), ("m2", 32), ("m3", 64)]
);
size!(abi_size_83, "struct S { unsigned int : 11; int m9; };", "struct S", 8);
bits!(abi_bits_83, "struct S { unsigned int : 11; int m9; };", "S", &[("m9", 32)]);
size!(
    abi_size_84,
    "union I84_0 { float f0; short f1; }; struct S { short m0[3]; int : 2; int : 12; float m3[1]; signed char m4; signed char m5; };",
    "struct S",
    16
);
bits!(
    abi_bits_84,
    "union I84_0 { float f0; short f1; }; struct S { short m0[3]; int : 2; int : 12; float m3[1]; signed char m4; signed char m5; };",
    "S",
    &[("m0", 0), ("m3", 64), ("m4", 96), ("m5", 104)]
);
size!(
    abi_size_85,
    "union I85_0 { unsigned short f0; signed char f1; float f2; }; struct S { long double m0; unsigned long m1; unsigned long m2; char * m3[1]; int : 1; };",
    "struct S",
    28
);
bits!(
    abi_bits_85,
    "union I85_0 { unsigned short f0; signed char f1; float f2; }; struct S { long double m0; unsigned long m1; unsigned long m2; char * m3[1]; int : 1; };",
    "S",
    &[("m0", 0), ("m1", 96), ("m2", 128), ("m3", 160)]
);
size!(abi_size_86, "struct I86_0 { long double f0; }; struct S { unsigned int : 13; int m9; };", "struct S", 8);
bits!(abi_bits_86, "struct I86_0 { long double f0; }; struct S { unsigned int : 13; int m9; };", "S", &[("m9", 32)]);
size!(abi_size_87, "union I87_0 { double f0; }; union S { union I87_0 m0; };", "union S", 8);
bits!(abi_bits_87, "union I87_0 { double f0; }; union S { union I87_0 m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_88,
    "union I88_0 { unsigned short f0; signed char f1; }; union S { unsigned short m0[4]; int : 5; int : 6; unsigned short m3[1]; unsigned long m4[3]; };",
    "union S",
    12
);
bits!(
    abi_bits_88,
    "union I88_0 { unsigned short f0; signed char f1; }; union S { unsigned short m0[4]; int : 5; int : 6; unsigned short m3[1]; unsigned long m4[3]; };",
    "S",
    &[("m0", 0), ("m3", 0), ("m4", 0)]
);
size!(
    abi_size_89,
    "union I89_0 { char * f0; }; union S { short m0; unsigned long m1; double m2; void * m3; };",
    "union S",
    8
);
bits!(
    abi_bits_89,
    "union I89_0 { char * f0; }; union S { short m0; unsigned long m1; double m2; void * m3; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(
    abi_size_90,
    "union I90_0 { unsigned short f0; char f1; float f2; }; union S { unsigned int : 10; unsigned int m1 : 12; };",
    "union S",
    4
);
bits!(
    abi_bits_90,
    "union I90_0 { unsigned short f0; char f1; float f2; }; union S { unsigned int : 10; unsigned int m1 : 12; };",
    "S",
    &[("m1", 0)]
);
size!(
    abi_size_91,
    "struct I91_0 { double f0; double f1; }; union S { int m0 : 30; int : 0; long m2; void * m3; unsigned int m4; unsigned long m5; };",
    "union S",
    4
);
bits!(
    abi_bits_91,
    "struct I91_0 { double f0; double f1; }; union S { int m0 : 30; int : 0; long m2; void * m3; unsigned int m4; unsigned long m5; };",
    "S",
    &[("m0", 0), ("m2", 0), ("m3", 0), ("m4", 0), ("m5", 0)]
);
size!(abi_size_92, "union S { int * m0[4]; };", "union S", 16);
bits!(abi_bits_92, "union S { int * m0[4]; };", "S", &[("m0", 0)]);
size!(
    abi_size_93,
    "union I93_0 { unsigned char f0; unsigned int f1; }; struct S { unsigned char m0[1]; int : 0; unsigned char m2; unsigned int m3 : 23; long double m4; union I93_0 m5; };",
    "struct S",
    24
);
bits!(
    abi_bits_93,
    "union I93_0 { unsigned char f0; unsigned int f1; }; struct S { unsigned char m0[1]; int : 0; unsigned char m2; unsigned int m3 : 23; long double m4; union I93_0 m5; };",
    "S",
    &[("m0", 0), ("m2", 32), ("m3", 40), ("m4", 64), ("m5", 160)]
);
size!(abi_size_94, "union S { char m0; int m1; signed int : 2; void * m3; unsigned int m4 : 5; };", "union S", 4);
bits!(
    abi_bits_94,
    "union S { char m0; int m1; signed int : 2; void * m3; unsigned int m4 : 5; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m3", 0), ("m4", 0)]
);
size!(
    abi_size_95,
    "struct S { int : 8; unsigned char m1[3]; unsigned int : 15; int m3[3]; short m4[2]; unsigned char m5; };",
    "struct S",
    28
);
bits!(
    abi_bits_95,
    "struct S { int : 8; unsigned char m1[3]; unsigned int : 15; int m3[3]; short m4[2]; unsigned char m5; };",
    "S",
    &[("m1", 8), ("m3", 64), ("m4", 160), ("m5", 192)]
);
size!(
    abi_size_96,
    "union I96_0 { int * f0; double f1; }; union S { unsigned char m0; int m1 : 3; unsigned long m2[2]; signed int : 16; signed int m4 : 30; void * m5; };",
    "union S",
    8
);
bits!(
    abi_bits_96,
    "union I96_0 { int * f0; double f1; }; union S { unsigned char m0; int m1 : 3; unsigned long m2[2]; signed int : 16; signed int m4 : 30; void * m5; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m4", 0), ("m5", 0)]
);
size!(abi_size_97, "struct S { int m0 : 23; unsigned short m1; signed int : 3; };", "struct S", 8);
bits!(abi_bits_97, "struct S { int m0 : 23; unsigned short m1; signed int : 3; };", "S", &[("m0", 0), ("m1", 32)]);
size!(abi_size_98, "struct I98_0 { void * f0; }; struct S { int : 0; char * m1; };", "struct S", 4);
bits!(abi_bits_98, "struct I98_0 { void * f0; }; struct S { int : 0; char * m1; };", "S", &[("m1", 0)]);
size!(abi_size_99, "struct I99_0 { int * f0; short f1; }; struct S { char m0[3]; };", "struct S", 3);
bits!(abi_bits_99, "struct I99_0 { int * f0; short f1; }; struct S { char m0[3]; };", "S", &[("m0", 0)]);
size!(
    abi_size_100,
    "struct I100_0 { int f0; unsigned int f1; unsigned int f2; }; union S { double m0; float m1; char m2; unsigned int m3 : 18; };",
    "union S",
    8
);
bits!(
    abi_bits_100,
    "struct I100_0 { int f0; unsigned int f1; unsigned int f2; }; union S { double m0; float m1; char m2; unsigned int m3 : 18; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0)]
);
size!(abi_size_101, "struct S { long m0; float m1[1]; };", "struct S", 8);
bits!(abi_bits_101, "struct S { long m0; float m1[1]; };", "S", &[("m0", 0), ("m1", 32)]);
size!(abi_size_102, "struct S { unsigned int : 11; int : 2; unsigned short m2; int : 0; };", "struct S", 4);
bits!(abi_bits_102, "struct S { unsigned int : 11; int : 2; unsigned short m2; int : 0; };", "S", &[("m2", 16)]);
size!(abi_size_103, "struct S { float m0; signed char m1[4]; int * m2; int : 14; unsigned int : 4; };", "struct S", 16);
bits!(
    abi_bits_103,
    "struct S { float m0; signed char m1[4]; int * m2; int : 14; unsigned int : 4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64)]
);
size!(
    abi_size_104,
    "union I104_0 { int * f0; }; struct S { unsigned int : 2; long m1; int * m2[2]; unsigned int : 6; union I104_0 m4; };",
    "struct S",
    24
);
bits!(
    abi_bits_104,
    "union I104_0 { int * f0; }; struct S { unsigned int : 2; long m1; int * m2[2]; unsigned int : 6; union I104_0 m4; };",
    "S",
    &[("m1", 32), ("m2", 64), ("m4", 160)]
);
size!(
    abi_size_105,
    "union I105_0 { unsigned char f0; }; struct S { unsigned int m0; long m1; unsigned long m2; void * m3; void * m4[1]; };",
    "struct S",
    20
);
bits!(
    abi_bits_105,
    "union I105_0 { unsigned char f0; }; struct S { unsigned int m0; long m1; unsigned long m2; void * m3; void * m4[1]; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96), ("m4", 128)]
);
size!(abi_size_106, "union S { int m0; long double m1; };", "union S", 12);
bits!(abi_bits_106, "union S { int m0; long double m1; };", "S", &[("m0", 0), ("m1", 0)]);
size!(
    abi_size_107,
    "struct I107_0 { int f0; void * f1; }; struct S { unsigned int : 6; unsigned long m1; double m2; unsigned int m3 : 18; unsigned long m4; };",
    "struct S",
    24
);
bits!(
    abi_bits_107,
    "struct I107_0 { int f0; void * f1; }; struct S { unsigned int : 6; unsigned long m1; double m2; unsigned int m3 : 18; unsigned long m4; };",
    "S",
    &[("m1", 32), ("m2", 64), ("m3", 128), ("m4", 160)]
);
size!(
    abi_size_108,
    "struct I108_0 { long double f0; signed char f1; void * f2; }; struct S { unsigned int m0 : 10; };",
    "struct S",
    4
);
bits!(
    abi_bits_108,
    "struct I108_0 { long double f0; signed char f1; void * f2; }; struct S { unsigned int m0 : 10; };",
    "S",
    &[("m0", 0)]
);
size!(
    abi_size_109,
    "struct I109_0 { unsigned int f0; int * f1; unsigned int f2; }; struct S { struct I109_0 m0; unsigned int m1 : 27; };",
    "struct S",
    16
);
bits!(
    abi_bits_109,
    "struct I109_0 { unsigned int f0; int * f1; unsigned int f2; }; struct S { struct I109_0 m0; unsigned int m1 : 27; };",
    "S",
    &[("m0", 0), ("m1", 96)]
);
size!(
    abi_size_110,
    "struct I110_0 { short f0; signed char f1; int f2; }; struct S { unsigned int m0; unsigned char m1; int m2 : 3; struct I110_0 m3; char m4; };",
    "struct S",
    20
);
bits!(
    abi_bits_110,
    "struct I110_0 { short f0; signed char f1; int f2; }; struct S { unsigned int m0; unsigned char m1; int m2 : 3; struct I110_0 m3; char m4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 40), ("m3", 64), ("m4", 128)]
);
size!(abi_size_111, "struct S { int : 0; unsigned int m1 : 6; int : 3; };", "struct S", 4);
bits!(abi_bits_111, "struct S { int : 0; unsigned int m1 : 6; int : 3; };", "S", &[("m1", 0)]);
size!(abi_size_112, "struct S { int m0; };", "struct S", 4);
bits!(abi_bits_112, "struct S { int m0; };", "S", &[("m0", 0)]);
size!(abi_size_113, "struct S { unsigned int m0; unsigned int m1; unsigned int : 10; int m3 : 19; };", "struct S", 12);
bits!(
    abi_bits_113,
    "struct S { unsigned int m0; unsigned int m1; unsigned int : 10; int m3 : 19; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m3", 74)]
);
size!(
    abi_size_114,
    "union I114_0 { int * f0; unsigned int f1; }; union S { unsigned short m0; signed int m1 : 14; unsigned int m2 : 27; };",
    "union S",
    4
);
bits!(
    abi_bits_114,
    "union I114_0 { int * f0; unsigned int f1; }; union S { unsigned short m0; signed int m1 : 14; unsigned int m2 : 27; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0)]
);
size!(
    abi_size_115,
    "struct S { unsigned char m0; unsigned long m1; signed int m2 : 9; double m3; unsigned int : 5; };",
    "struct S",
    24
);
bits!(
    abi_bits_115,
    "struct S { unsigned char m0; unsigned long m1; signed int m2 : 9; double m3; unsigned int : 5; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96)]
);
size!(abi_size_116, "struct I116_0 { long double f0; }; union S { int * m0; int m1; };", "union S", 4);
bits!(abi_bits_116, "struct I116_0 { long double f0; }; union S { int * m0; int m1; };", "S", &[("m0", 0), ("m1", 0)]);
size!(abi_size_117, "union S { int m0; float m1; signed int : 14; };", "union S", 4);
bits!(abi_bits_117, "union S { int m0; float m1; signed int : 14; };", "S", &[("m0", 0), ("m1", 0)]);
size!(abi_size_118, "struct S { char m0[4]; unsigned int m1; };", "struct S", 8);
bits!(abi_bits_118, "struct S { char m0[4]; unsigned int m1; };", "S", &[("m0", 0), ("m1", 32)]);
size!(
    abi_size_119,
    "union I119_0 { unsigned int f0; int f1; unsigned long f2; }; struct S { signed char m0; long double m1; unsigned int m2; int : 0; union I119_0 m4; unsigned int m5; };",
    "struct S",
    28
);
bits!(
    abi_bits_119,
    "union I119_0 { unsigned int f0; int f1; unsigned long f2; }; struct S { signed char m0; long double m1; unsigned int m2; int : 0; union I119_0 m4; unsigned int m5; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 128), ("m4", 160), ("m5", 192)]
);
size!(
    abi_size_120,
    "struct I120_0 { short f0; }; struct S { unsigned char m0; signed char m1; struct I120_0 m2; };",
    "struct S",
    4
);
bits!(
    abi_bits_120,
    "struct I120_0 { short f0; }; struct S { unsigned char m0; signed char m1; struct I120_0 m2; };",
    "S",
    &[("m0", 0), ("m1", 8), ("m2", 16)]
);
size!(abi_size_121, "union I121_0 { short f0; signed char f1; }; struct S { void * m0; int m1; };", "struct S", 8);
bits!(
    abi_bits_121,
    "union I121_0 { short f0; signed char f1; }; struct S { void * m0; int m1; };",
    "S",
    &[("m0", 0), ("m1", 32)]
);
size!(abi_size_122, "struct S { unsigned int : 14; unsigned int m1; signed int : 13; };", "struct S", 12);
bits!(abi_bits_122, "struct S { unsigned int : 14; unsigned int m1; signed int : 13; };", "S", &[("m1", 32)]);
size!(abi_size_123, "struct S { void * m0; char m1; short m2; unsigned int m3 : 25; int : 4; };", "struct S", 12);
bits!(
    abi_bits_123,
    "struct S { void * m0; char m1; short m2; unsigned int m3 : 25; int : 4; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 48), ("m3", 64)]
);
size!(abi_size_124, "union S { void * m0; };", "union S", 4);
bits!(abi_bits_124, "union S { void * m0; };", "S", &[("m0", 0)]);
size!(
    abi_size_125,
    "struct I125_0 { unsigned long f0; unsigned int f1; long double f2; }; union S { struct I125_0 m0; };",
    "union S",
    20
);
bits!(
    abi_bits_125,
    "struct I125_0 { unsigned long f0; unsigned int f1; long double f2; }; union S { struct I125_0 m0; };",
    "S",
    &[("m0", 0)]
);
size!(
    abi_size_126,
    "struct I126_0 { int * f0; unsigned long f1; char f2; }; struct S { unsigned int m0 : 9; int m1 : 6; };",
    "struct S",
    4
);
bits!(
    abi_bits_126,
    "struct I126_0 { int * f0; unsigned long f1; char f2; }; struct S { unsigned int m0 : 9; int m1 : 6; };",
    "S",
    &[("m0", 0), ("m1", 9)]
);
size!(abi_size_127, "struct I127_0 { char f0; char f1; }; struct S { int m0 : 27; };", "struct S", 4);
bits!(abi_bits_127, "struct I127_0 { char f0; char f1; }; struct S { int m0 : 27; };", "S", &[("m0", 0)]);
size!(abi_size_128, "struct S { unsigned short m0; int : 6; unsigned long m2; };", "struct S", 8);
bits!(abi_bits_128, "struct S { unsigned short m0; int : 6; unsigned long m2; };", "S", &[("m0", 0), ("m2", 32)]);
size!(abi_size_129, "union S { void * m0[4]; short m1; };", "union S", 16);
bits!(abi_bits_129, "union S { void * m0[4]; short m1; };", "S", &[("m0", 0), ("m1", 0)]);
size!(abi_size_130, "struct S { unsigned int m0; int m1 : 25; int : 0; };", "struct S", 8);
bits!(abi_bits_130, "struct S { unsigned int m0; int m1 : 25; int : 0; };", "S", &[("m0", 0), ("m1", 32)]);
size!(abi_size_131, "struct S { int : 6; int : 0; unsigned int m2 : 18; };", "struct S", 8);
bits!(abi_bits_131, "struct S { int : 6; int : 0; unsigned int m2 : 18; };", "S", &[("m2", 32)]);
size!(
    abi_size_132,
    "union I132_0 { void * f0; }; struct S { union I132_0 m0; int : 2; unsigned int m2 : 16; int * m3; short m4; int m5; };",
    "struct S",
    20
);
bits!(
    abi_bits_132,
    "union I132_0 { void * f0; }; struct S { union I132_0 m0; int : 2; unsigned int m2 : 16; int * m3; short m4; int m5; };",
    "S",
    &[("m0", 0), ("m2", 34), ("m3", 64), ("m4", 96), ("m5", 128)]
);
size!(
    abi_size_133,
    "union I133_0 { signed char f0; }; struct S { int m0 : 18; unsigned int m1; signed int m2 : 18; long m3[4]; };",
    "struct S",
    28
);
bits!(
    abi_bits_133,
    "union I133_0 { signed char f0; }; struct S { int m0 : 18; unsigned int m1; signed int m2 : 18; long m3[4]; };",
    "S",
    &[("m0", 0), ("m1", 32), ("m2", 64), ("m3", 96)]
);
size!(abi_size_134, "struct S { int : 0; int m9; };", "struct S", 4);
bits!(abi_bits_134, "struct S { int : 0; int m9; };", "S", &[("m9", 0)]);
size!(abi_size_135, "struct I135_0 { short f0; }; struct S { short m0; };", "struct S", 2);
bits!(abi_bits_135, "struct I135_0 { short f0; }; struct S { short m0; };", "S", &[("m0", 0)]);
size!(abi_size_136, "struct S { signed int : 6; float m1; int m2 : 31; };", "struct S", 12);
bits!(abi_bits_136, "struct S { signed int : 6; float m1; int m2 : 31; };", "S", &[("m1", 32), ("m2", 64)]);
size!(
    abi_size_137,
    "union I137_0 { int * f0; char f1; unsigned int f2; }; union S { signed char m0[4]; };",
    "union S",
    4
);
bits!(
    abi_bits_137,
    "union I137_0 { int * f0; char f1; unsigned int f2; }; union S { signed char m0[4]; };",
    "S",
    &[("m0", 0)]
);
size!(
    abi_size_138,
    "union S { signed int m0 : 1; unsigned char m1[1]; double m2; void * m3; int m4; signed char m5; };",
    "union S",
    8
);
bits!(
    abi_bits_138,
    "union S { signed int m0 : 1; unsigned char m1[1]; double m2; void * m3; int m4; signed char m5; };",
    "S",
    &[("m0", 0), ("m1", 0), ("m2", 0), ("m3", 0), ("m4", 0), ("m5", 0)]
);
size!(
    abi_size_139,
    "struct S { int : 15; unsigned long m1; unsigned int : 13; signed char m3; char m4; };",
    "struct S",
    12
);
bits!(
    abi_bits_139,
    "struct S { int : 15; unsigned long m1; unsigned int : 13; signed char m3; char m4; };",
    "S",
    &[("m1", 32), ("m3", 80), ("m4", 88)]
);
