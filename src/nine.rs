const WORKSPACE_MAP: [(i32, Position); 9] = [
    (2, TopLeft),
    (1, MiddleLeft),
    (7, BottomLeft),
    (3, TopMiddle),
    (5, MiddleMiddle),
    (8, BottomMiddle),
    (4, TopRight),
    (6, MiddleRight),
    (0, BottomRight),
];

#[derive(Debug)]
pub enum NineCmd {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MovedTo(i32),
}

use NineCmd::*;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Position {
    #[default]
    TopLeft,
    MiddleLeft,
    BottomLeft,
    TopMiddle,
    MiddleMiddle,
    BottomMiddle,
    TopRight,
    MiddleRight,
    BottomRight,
}

use Position::*;

impl From<i32> for Position {
    fn from(value: i32) -> Self {
        let name = WORKSPACE_MAP.iter().find_map(
            |(num, pos)| {
                if *num == value {
                    Some(pos)
                } else {
                    None
                }
            },
        );

        if let Some(name) = name {
            *name
        } else {
            panic!()
        }
    }
}

/*
impl From<String> for Position {
    fn from(value: String) -> Self {
        match value.as_str() {
            "TopLeft" => TopLeft,
            "MiddleLeft" => MiddleLeft,
            "BottomLeft" => BottomLeft,
            "TopMiddle" => TopMiddle,
            "MiddleMiddle" => MiddleMiddle,
            "BottomMiddle" => BottomMiddle,
            "TopRight" => TopRight,
            "MiddleRight" => MiddleRight,
            "BottomRight" => BottomRight,
            _ => panic!(),
        }
    }
}
*/

impl Position {
    pub fn num(&self) -> i32 {
        let num =
            WORKSPACE_MAP
                .iter()
                .find_map(|(num, pos)| if pos == self { Some(num) } else { None });

        if let Some(num) = num {
            *num
        } else {
            panic!()
        }
    }

    /*
    pub fn name(&self) -> &'static str {
        match self {
            TopLeft => "TopLeft",
            MiddleLeft => "MiddleLeft",
            BottomLeft => "BottomLeft",
            TopMiddle => "TopMiddle",
            MiddleMiddle => "MiddleMiddle",
            BottomMiddle => "BottomMiddle",
            TopRight => "TopRight",
            MiddleRight => "MiddleRight",
            BottomRight => "BottomRight",
        }
    }
    */

    pub fn map_cmd(&self, cmd: NineCmd) -> Self {
        let grid = [
            [TopLeft, MiddleLeft, BottomLeft],
            [TopMiddle, MiddleMiddle, BottomMiddle],
            [TopRight, MiddleRight, BottomRight],
        ];
        let [col, row] = match self {
            TopLeft => [0, 0],
            MiddleLeft => [0, 1],
            BottomLeft => [0, 2],
            TopMiddle => [1, 0],
            MiddleMiddle => [1, 1],
            BottomMiddle => [1, 2],
            TopRight => [2, 0],
            MiddleRight => [2, 1],
            BottomRight => [2, 2],
        };

        match cmd {
            MoveLeft => grid[(col + 2) % 3][row],
            MoveRight => grid[(col + 1) % 3][row],
            MoveUp => grid[col][(row + 2) % 3],
            MoveDown => grid[col][(row + 1) % 3],
            MovedTo(_) => panic!(),
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ⮰ ⭦
        f.write_str(match self {
            TopLeft => "T__",
            MiddleLeft => "M__",
            BottomLeft => "B__",
            TopMiddle => "_T_",
            MiddleMiddle => "_M_",
            BottomMiddle => "_B_",
            TopRight => "__T",
            MiddleRight => "__M",
            BottomRight => "__B",
        })
    }
}
