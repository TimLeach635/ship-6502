use bevy::prelude::Component;

#[derive(Component)]
pub struct Computer;

impl Computer {
    pub fn get_screen(&self) -> &str {
        "A quick brown fox jumps over the lazy dog.
0123456789 ¿?¡!`'\"., <>()[]{} &@%*^#$\\/

* Wieniläinen sioux'ta puhuva ökyzombie diggaa Åsan roquefort-tacoja.
* Ça me fait peur de fêter noël là, sur cette île bizarroïde où une mère et sa
môme essaient de me tuer avec un gâteau à la cigüe brûlé.
* Zwölf Boxkämpfer jagten Eva quer über den Sylter Deich.
* El pingüino Wenceslao hizo kilómetros bajo exhaustiva lluvia y frío, añoraba
a su querido cachorro.

┌─┬─┐ ╔═╦═╗ ╒═╤═╕ ╓─╥─╖
│ │ │ ║ ║ ║ │ │ │ ║ ║ ║
├─┼─┤ ╠═╬═╣ ╞═╪═╡ ╟─╫─╢
└─┴─┘ ╚═╩═╝ ╘═╧═╛ ╙─╨─╜

░░░░░ ▐▀█▀▌ .·∙•○°○•∙·.
▒▒▒▒▒ ▐ █ ▌ ☺☻ ♥♦♣♠ ♪♫☼
▓▓▓▓▓ ▐▀█▀▌  $ ¢ £ ¥ ₧
█████ ▐▄█▄▌ ◄►▲▼ ←→↑↓↕↨

⌠
│dx ≡ Σ √x²ⁿ·δx
⌡"
    }
}
