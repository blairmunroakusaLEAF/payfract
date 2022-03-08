/****************************************************************
 * Fracpay server CreateMAIN instruction process 
 * blairmunroakusa@.0322.anch.AK			    
 ****************************************************************/

#![allow(non_snake_case)]
use solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
        msg,
    };
use bit_vec::BitVec;
use crate::{
        processor::{
            run::Processor,
            utility::*,
        },
        state::{
            constants::*,
            MAIN::*,
            PIECE::*,
            REF::*,
        },
    };

impl Processor {

    pub fn process_create_main<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        bumpMAIN: u8,
        seedMAIN: Vec<u8>,
        bumpPIECE: u8,
        seedPIECE: Vec<u8>,
        bumpREF: u8,
        seedREF: Vec<u8>,
    ) -> ProgramResult {

        // get accounts
        let (operator, rent, pda) = get_accounts(accounts)?;

        // check to make sure tx operator is signer
        if !operator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // calculate rent
        let rentMAIN = rent.minimum_balance(SIZE_MAIN.into());
        let rentPIECE = rent.minimum_balance(SIZE_PIECE.into());
        let rentREF = rent.minimum_balance(SIZE_REF.into());
       
        // create pdaMAIN
        invoke_signed(
        &system_instruction::create_account(
            &operator.key,
            &pda.MAIN.key,
            rentMAIN,
            SIZE_MAIN.into(),
            &program_id
        ),
        &[
            operator.clone(),
            pda.MAIN.clone()
        ],
        &[&[&seedMAIN, &[bumpMAIN]]]
        )?;
        msg!("Successfully created pdaMAIN");

        // create pdaPIECEself
        invoke_signed(
        &system_instruction::create_account(
            &operator.key,
            &pda.PIECE.key,
            rentPIECE,
            SIZE_PIECE.into(),
            &program_id
        ),
        &[
            operator.clone(),
            pda.PIECE.clone()
        ],
        &[&[&seedPIECE, &[bumpPIECE]]]
        )?;
        msg!("Successfully created pdaPIECE");

        // create pdaREFself
        invoke_signed(
        &system_instruction::create_account(
            &operator.key,
            &pda.REF.key,
            rentREF,
            SIZE_REF.into(),
            program_id
        ),
        &[
            operator.clone(),
            pda.REF.clone()
        ],
        &[&[&seedREF, &[bumpREF]]]
        )?;
        msg!("Successfully created pdaREF");

        // get MAIN info
        let mut MAINinfo = MAIN::unpack_unchecked(&pda.MAIN.try_borrow_data()?)?;

        // set flags
        let mut FLAGS = BitVec::from_elem(16, false);
        FLAGS.set(0, false); // MAIN account is 0000
        FLAGS.set(1, false);
        FLAGS.set(2, false); 
        FLAGS.set(3, false); 

        // initialize MAIN account data
        MAINinfo.flags = pack_flags(FLAGS);
        MAINinfo.operator = *operator.key;
        MAINinfo.balance = 0;
        MAINinfo.netsum = 0;
        MAINinfo.piececount = 0;
        MAIN::pack(MAINinfo, &mut pda.MAIN.try_borrow_mut_data()?)?;

        // get PIECE info
        let mut PIECEinfo = PIECE::unpack_unchecked(&pda.PIECE.try_borrow_data()?)?;

        // set flags
        let mut FLAGS = BitVec::from_elem(16, false);
        FLAGS.set(0, false); // PIECE self account is 0001
        FLAGS.set(1, false);
        FLAGS.set(2, false); 
        FLAGS.set(3, true); 

        // initialize self PIECE account data
        PIECEinfo.flags = pack_flags(FLAGS);
        PIECEinfo.operator = *operator.key;
        PIECEinfo.balance = 0;
        PIECEinfo.netsum = 0;
        PIECEinfo.refcount = 0;
        PIECEinfo.pieceslug = pack_pieceslug(seedMAIN);
        PIECE::pack(PIECEinfo, &mut pda.PIECE.try_borrow_mut_data()?)?;

        // get REF info
        let mut REFinfo = REF::unpack_unchecked(&pda.REF.try_borrow_data()?)?;

        // set flags
        let mut FLAGS = BitVec::from_elem(16, false);
        FLAGS.set(0, false); // REF self account is 0010
        FLAGS.set(1, false);
        FLAGS.set(2, true);
        FLAGS.set(4, false);

        // initialize self REF account data
        REFinfo.flags = pack_flags(FLAGS);
        REFinfo.target = *operator.key;
        REFinfo.fract = 100_000_000;    // new self-ref gets 100% by default
        REFinfo.netsum = 0;
        REFinfo.refslug = pack_refslug("SELF-REFERENCE".as_bytes().to_vec());
        REF::pack(REFinfo, &mut pda.REF.try_borrow_mut_data()?)?;

        Ok(())
    }
}

