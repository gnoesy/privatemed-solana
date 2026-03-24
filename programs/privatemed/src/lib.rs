use anchor_lang::prelude::*;

declare_id!("PMed1111111111111111111111111111111111111111");

/// PrivateMed — Encrypted medical records and consent management via Arcium MXE
///
/// Patient data is encrypted with MXE public key. Medical computations
/// (diagnosis matching, drug interaction checks) run in the MXE.
/// Only access permissions and consent records land on-chain.
#[program]
pub mod privatemed {
    use super::*;

    pub fn register_patient(
        ctx: Context<RegisterPatient>,
        patient_id: u64,
        encrypted_health_record: Vec<u8>,
        record_commitment: [u8; 32],
        mxe_cluster_offset: u64,
    ) -> Result<()> {
        require!(encrypted_health_record.len() <= 1024, PrivateMedError::DataTooLarge);
        let patient = &mut ctx.accounts.patient;
        patient.owner = ctx.accounts.owner.key();
        patient.patient_id = patient_id;
        patient.encrypted_health_record = encrypted_health_record;
        patient.record_commitment = record_commitment;
        patient.mxe_cluster_offset = mxe_cluster_offset;
        patient.registered_at = Clock::get()?.unix_timestamp;
        emit!(PatientRegistered { patient_id, mxe_cluster_offset });
        Ok(())
    }

    pub fn grant_access(
        ctx: Context<GrantAccess>,
        patient_id: u64,
        provider: Pubkey,
        access_level: AccessLevel,
        expires_at: i64,
    ) -> Result<()> {
        let consent = &mut ctx.accounts.consent;
        consent.patient = ctx.accounts.patient_owner.key();
        consent.provider = provider;
        consent.patient_id = patient_id;
        consent.access_level = access_level;
        consent.expires_at = expires_at;
        consent.revoked = false;
        emit!(AccessGranted { patient_id, provider, access_level, expires_at });
        Ok(())
    }

    pub fn record_mxe_diagnosis(
        ctx: Context<RecordDiagnosis>,
        patient_id: u64,
        diagnosis_commitment: [u8; 32],
        mxe_proof_hash: [u8; 32],
    ) -> Result<()> {
        let record = &mut ctx.accounts.diagnosis_record;
        record.patient_id = patient_id;
        record.provider = ctx.accounts.provider.key();
        record.diagnosis_commitment = diagnosis_commitment;
        record.mxe_proof_hash = mxe_proof_hash;
        record.recorded_at = Clock::get()?.unix_timestamp;
        emit!(DiagnosisRecorded { patient_id, mxe_proof_hash });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(patient_id: u64)]
pub struct RegisterPatient<'info> {
    #[account(init, payer = owner, space = Patient::LEN,
        seeds = [b"patient", owner.key().as_ref()], bump)]
    pub patient: Account<'info, Patient>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(patient_id: u64)]
pub struct GrantAccess<'info> {
    #[account(init, payer = patient_owner, space = ConsentRecord::LEN,
        seeds = [b"consent", &patient_id.to_le_bytes(), patient_owner.key().as_ref()], bump)]
    pub consent: Account<'info, ConsentRecord>,
    #[account(mut)]
    pub patient_owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(patient_id: u64)]
pub struct RecordDiagnosis<'info> {
    #[account(init, payer = provider, space = DiagnosisRecord::LEN,
        seeds = [b"diag", &patient_id.to_le_bytes(), provider.key().as_ref()], bump)]
    pub diagnosis_record: Account<'info, DiagnosisRecord>,
    #[account(mut)]
    pub provider: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Patient {
    pub owner: Pubkey, pub patient_id: u64,
    pub encrypted_health_record: Vec<u8>,
    pub record_commitment: [u8; 32],
    pub mxe_cluster_offset: u64, pub registered_at: i64,
}
impl Patient { pub const LEN: usize = 8 + 32 + 8 + (4+1024) + 32 + 8 + 8; }

#[account]
pub struct ConsentRecord {
    pub patient: Pubkey, pub provider: Pubkey, pub patient_id: u64,
    pub access_level: AccessLevel, pub expires_at: i64, pub revoked: bool,
}
impl ConsentRecord { pub const LEN: usize = 8 + 32 + 32 + 8 + 1 + 8 + 1; }

#[account]
pub struct DiagnosisRecord {
    pub patient_id: u64, pub provider: Pubkey,
    pub diagnosis_commitment: [u8; 32], pub mxe_proof_hash: [u8; 32], pub recorded_at: i64,
}
impl DiagnosisRecord { pub const LEN: usize = 8 + 8 + 32 + 32 + 32 + 8; }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AccessLevel { ReadOnly, DiagnosisOnly, Full }

#[event]
pub struct PatientRegistered { pub patient_id: u64, pub mxe_cluster_offset: u64 }
#[event]
pub struct AccessGranted { pub patient_id: u64, pub provider: Pubkey, pub access_level: AccessLevel, pub expires_at: i64 }
#[event]
pub struct DiagnosisRecorded { pub patient_id: u64, pub mxe_proof_hash: [u8; 32] }

#[error_code]
pub enum PrivateMedError {
    #[msg("Data exceeds size limit")]
    DataTooLarge,
}
