use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Production {
    CreatureP(CreatureID),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Building {
    pub recipe_book: Recipe,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Recipe {
    pub produces: Production,
    pub duration: Ticks,
}

impl Building {
    pub fn new(recipe_book: RecipeBook) -> Building {
        Buidling { 
            recipe_book: recipe_book
        }
    }
}

impl Recipe {
    pub fn new(production: Production, duration: Duration) -> Recipe {
        Recipe {
            produces: production,
            duration: duration,
        }
    }
}

pub type RecipeBook = Recipe;
pub type Blueprints = Building;

pub fn init_recipes() -> RecipeBook {
    let recipe = Recipe::new(Production::CreatureP(1), 200);
    recipe
}

pub fn init_buildings -> Buildings {
    Buidling::new(init_recipes())
}
