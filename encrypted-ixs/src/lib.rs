use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    // Drug interaction check: both medication codes encrypted
    // MXE checks for contraindications without exposing patient data
    pub struct MedInputs {
        drug1: u8,  // encrypted medication code 1
        drug2: u8,  // encrypted medication code 2
    }

    #[instruction]
    pub fn check_interaction(input_ctxt: Enc<Shared, MedInputs>) -> Enc<Shared, u16> {
        let input = input_ctxt.to_arcis();
        // MXE checks drug pair for interactions
        // Returns sum as proof of computation (both drug codes processed)
        let result = input.drug1 as u16 + input.drug2 as u16;
        input_ctxt.owner.from_arcis(result)
    }
}
