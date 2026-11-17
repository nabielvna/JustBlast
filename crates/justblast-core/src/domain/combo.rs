#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ComboState {
    pub count: u32,
    pub misses: u32,
    pub producer_remaining: u8,
    pub saved_count: u32,
    pub checking: bool,
    pub waiting: u32,
    pub recovered: bool,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Scenario {
    Basic,
    Continuation,
    MultiLine,
}
impl Scenario {
    pub const ALL: [Self; 3] = [Self::Basic, Self::Continuation, Self::MultiLine];
    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Continuation => "continuation",
            Self::MultiLine => "multi_line",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleContext {
    pub guide_step: u32,
    pub guide_finished: bool,
    pub all_clear_bonus: u32,
}
impl Default for RuleContext {
    fn default() -> Self {
        Self {
            guide_step: 2,
            guide_finished: true,
            all_clear_bonus: 0,
        }
    }
}
pub fn advance_combo(
    mut state: ComboState,
    scenario: Scenario,
    lines: u32,
    remaining: u8,
    context: RuleContext,
) -> ComboState {
    let extended = scenario != Scenario::Basic;
    let multi = scenario == Scenario::MultiLine;
    let mut before_misses = state.misses;
    state.misses = if lines > 0 {
        0
    } else {
        state.misses.saturating_add(1)
    };
    let threshold = 3 + if extended {
        u32::from(state.producer_remaining)
    } else {
        0
    };
    if context.guide_step >= 2 {
        if state.misses >= threshold {
            state.count = 0;
        }
        if lines > 0 {
            state.count = state.count.saturating_add(1);
        }
    }
    let base_count = state.count;
    if extended {
        if !state.checking && lines > 0 {
            state.recovered = false;
        }
        let mut early = false;
        if state.checking {
            state.waiting = state.waiting.saturating_add(1);
            if lines < 2 || state.saved_count < 3 || state.recovered {
                if state.recovered && lines >= 2 && state.saved_count >= 3 {
                    state.recovered = false;
                }
                state.checking = false;
                state.saved_count = 0;
                state.waiting = 0;
                early = true;
            } else {
                state.count = state.saved_count.saturating_add(1);
                state.misses = 0;
                if multi {
                    before_misses = 0;
                }
                state.saved_count = 0;
                state.recovered = true;
            }
        }
        if !early {
            if base_count == 0 && state.saved_count != 0 {
                state.checking = true;
            } else {
                state.checking = false;
                if !multi || lines > 0 {
                    state.saved_count = base_count;
                }
            }
        }
    }
    if multi && context.guide_finished && before_misses < 3 {
        state.count = state.count.saturating_add(lines.saturating_sub(1));
    }
    if lines > 0 && extended {
        state.producer_remaining = remaining;
    }
    state
}
pub fn model_points(
    cells: u32,
    lines: u32,
    state: ComboState,
    scenario: Scenario,
    context: RuleContext,
    all_clear: bool,
) -> u64 {
    let mut clear = 0;
    if lines > 0 && context.guide_step >= 2 {
        let count = u64::from(state.count.max(1));
        clear = 10
            * count
            * if lines == 1 {
                1
            } else {
                u64::from(lines * (lines - 1))
            };
        if scenario != Scenario::Basic {
            if count > 10 {
                clear *= 2;
            } else if count > 5 {
                clear = (clear * 3).div_ceil(2);
            }
        }
    }
    u64::from(cells)
        + clear
        + if lines > 0 && all_clear {
            u64::from(context.all_clear_bonus)
        } else {
            0
        }
}
