// Runs at the head of `Game::native_use_item_on` (API 13): a construction
// stick claims the click and fills the clicked face's plane.
fn observe_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::constructionstick_state::on_use_on(game, stack, pos, face, claimed);
}
