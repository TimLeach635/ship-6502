# Design decision: how to implement the virtual computer

I keep going back and forth on what structure to implement for the virtual computer in the game.

I started by trying to code up a 6502 emulation, but it proved too complicated to get an OS loaded onto it. Next, I tried to write something simpler: a terminal that accepts input and prints lines to the virtual screen, and the internal "OS" (not an accurate name, but it's what I chose) that handles the commands and returns their output.

But I was quite excited at the prospect of having a very visual layout for the virtual computer, with nice borders and boxes everywhere – and under the system I've just written, I'd have to basically re-invent all the previous few decades' worth of development in the area of vaguely graphical command-line interfaces.

So, the "right" step is probably a simplified computer interface that has no knowledge of bytes and memory and terminals and stuff, and just draws strings onto the screen.

I'm definitely going to hang on to the terminal code, though – the byte array would come in really handy if, later, I want to move back to a 6502-based emulation.

That is, if I can stop Rust yelling at me about the unused code.
