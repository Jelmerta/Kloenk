use std::collections::HashMap;

#[derive(Clone)]
pub struct Dialogue {
    pub text: String,
}

impl Dialogue {}

// Probably at some point want to store this in file and load
pub struct DialogueManager {
    dialogues: HashMap<String, Dialogue>,
}

impl DialogueManager {
    pub fn new() -> DialogueManager {
        Self {
            dialogues: Self::load_dialogues(),
        }
    }

    fn load_dialogues() -> HashMap<String, Dialogue> {
        let mut dialogues: HashMap<String, Dialogue> = HashMap::new();

        dialogues.insert(
            "dennis_intro".to_string(),
            Dialogue {
                text: "Welcome to the exciting world of Kloenk!".to_string(),
            },
        );

        dialogues
    }

    pub fn get_dialogue(&self, id: &str) -> Option<&Dialogue> {
        self.dialogues.get(id)
    }
}
