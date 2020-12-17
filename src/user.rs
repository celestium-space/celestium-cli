use crate::transaction::TransactionValue;
use openssl::{ec::EcKey, pkey::Public};

pub struct User {
    pk: PublicKey,
    current_balance: u32,
    ids: Vec<u32>,
}

impl User {
    pub fn new(pk: PublicKey) -> User {
        User {
            pk,
            current_balance: 0,
            ids: Vec::new(),
        }
    }

    pub fn get_balance(&self) -> u32 {
        self.current_balance
    }

    pub fn give(&mut self, value: TransactionValue) -> Result<bool, String> {
        if value.is_coin_transfer()? {
            self.current_balance + value.get_value()?;
        } else {
            let tmp_id = value.get_id()?;
            if !self.ids.contains(&tmp_id) {
                self.ids.push(tmp_id);
            } else {
                return Err(format!(
                    "Trying to give user with pk {:?} the ID {}, which they already own",
                    self.pk, tmp_id
                ));
            }
        }
        return Ok(true);
    }
    pub fn take(&mut self, value: TransactionValue) -> Result<bool, String> {
        if value.is_coin_transfer()? {
            let tmp_value = value.get_value()?;
            if tmp_value <= self.current_balance {
                self.current_balance -= tmp_value;
            } else {
                return Err(format!("Trying to take {} from user with pk {:?}. This would make their balance negative ({})", tmp_value, self.pk, self.current_balance as i32 - tmp_value as i32));
            }
        } else {
            let tmp_id = value.get_id()?;
            if self.ids.contains(&tmp_id) {
                self.ids.push(tmp_id);
            } else {
                return Err(format!(
                    "Trying to take unowned id {} from user with pk {:?}",
                    tmp_id, self.pk,
                ));
            }
        }
        return Ok(true);
    }
}
