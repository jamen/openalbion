//! Fable's animation format. Probably the weakest understood format.

pub mod decode;
pub mod encode;

// Temporary comments from fabletlcmod.com.
//
// 3DAF: 3D Sequence File
//     ANRT: (File Size of all Chunks)
//     AOBJ: Animated Object
//         AMSK: Animation Mask
//         XSEQ: Compressed Animation Sequence
//     HLPR: Helpers
//         TMEV: Timing Event
//         MVEC: Movement Vector
//         XALO: Allocation Size Helper Chunk??
//
//
// 0 - No bytes (My personal favorite)
// 1 - 2 Ints (No Strange Stuff)
// 2 - INT, Byte, Byte (flag)
//                    Flag = 1: Int, Byte, Float, Int
//                    Flag = 0: Short, Int Int
// 3 - Int, Int, Int, Int (Flag?)
//                    Flag = 6: Byte, Byte
//                    Flag = 3: Int Int
//                    Flag = anything else?: Int
// 4 - Int, Int, Int, Int, Int, Int, Int (Flag)
//                    Flag = 0: Int
//                    Flag = 4: Int, Int
//                    Flag = 5: Int
//                    Flag = anything else?: No further data
// 5 - Int, Int, Int, Int, Int, Byte, Byte (Flag)
//                    Flag = 1: Int, Byte, Float, Float, Int, Int (String Size), ~String
//                    Flag = 0: Short, Int, Int, Int (Flag?)
//                                        Flag = 4294966519: Int, Int, Byte, Byte
//                                        Flag !=4294966519: Int, Int, Int
//
// I believe I cracked the format for animations a while ago. If I remember correctly it was a quaternion based system.
//
// Lets start it off simple. CAppearanceDef application, first off what does this do? It basically is the list of animations the hero users when called upon. This is essentially how I created the "Piss" Expression, you could change animation IDs for say when your hero runs with a large sword, add jumps etc... This Cdef couldn't be done with the xml so something custom was made.

pub struct Bba {
}