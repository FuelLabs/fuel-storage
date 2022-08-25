#![no_std]

mod impls;

extern crate alloc;

use alloc::borrow::Cow;

/// Merkle root alias type
pub type MerkleRoot = [u8; 32];

/// Mappable type with `Key` and `Value`.
pub trait Mappable {
    /// The type of the value's key.
    type Key;
    /// The value type is used while setting the value to the storage. In most cases, it is the same
    /// as `Self::GetValue`, but it is without restriction and can be used for performance
    /// optimizations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use core::marker::PhantomData;
    /// use fuel_storage::Mappable;
    /// pub struct Contract<'a>(PhantomData<&'a ()>);
    ///
    /// impl<'a> Mappable for Contract<'a> {
    ///     type Key = &'a [u8; 32];
    ///     /// It is optimized to use slice instead of vector.
    ///     type SetValue = [u8];
    ///     type GetValue = Vec<u8>;
    /// }
    /// ```
    type SetValue: ?Sized;
    /// The value type is used while getting the value from the storage.
    type GetValue: Clone;
}

/// Trait describes used errors during work with `Storage` for the `Type`.
pub trait StorageError<Type: Mappable> {
    type Error;
}

/// Base read storage trait for Fuel infrastructure.
pub trait StorageInspect<Type: Mappable>: StorageError<Type> {
    /// Retrieve `Cow<Value>` such as `Key->Value`.
    fn get(&self, key: &Type::Key) -> Result<Option<Cow<Type::GetValue>>, Self::Error>;

    /// Return `true` if there is a `Key` mapping to a value in the storage.
    fn contains_key(&self, key: &Type::Key) -> Result<bool, Self::Error>;
}

/// Base write storage trait for Fuel infrastructure.
pub trait StorageMutate<Type: Mappable>: StorageError<Type> {
    /// Append `Key->Value` mapping to the storage.
    ///
    /// If `Key` was already mappped to a value, return the replaced value as `Ok(Some(Value))`. Return
    /// `Ok(None)` otherwise.
    fn insert(
        &mut self,
        key: &Type::Key,
        value: &Type::SetValue,
    ) -> Result<Option<Type::GetValue>, Self::Error>;

    /// Remove `Key->Value` mapping from the storage.
    ///
    /// Return `Ok(Some(Value))` if the value was present. If the key wasn't found, return
    /// `Ok(None)`.
    fn remove(&mut self, key: &Type::Key) -> Result<Option<Type::GetValue>, Self::Error>;
}

/// Base storage trait for Fuel infrastructure.
///
/// Generic should implement [`Mappable`](crate::Mappable) trait with all storage type information.
pub trait Storage<Type: Mappable>: StorageMutate<Type> + StorageInspect<Type> {}

/// Returns the merkle root for the `StorageType` per merkle `Key`. The type should implement the
/// `Storage` for the `StorageType`. Per one storage, it is possible to have several merkle trees
/// under different `Key`.
pub trait MerkleRootStorage<Key, StorageType>: Storage<StorageType>
where
    StorageType: Mappable,
{
    /// Return the merkle root of the stored `Type` in the `Storage`.
    ///
    /// The cryptographic primitive is an arbitrary choice of the implementor and this trait won't
    /// impose any restrictions to that.
    fn root(&mut self, key: &Key) -> Result<MerkleRoot, Self::Error>;
}

/// The wrapper around the `Storage` that supports only methods from `StorageInspect`.
pub struct StorageRef<'a, T: 'a + ?Sized, Type: Mappable>(&'a T, core::marker::PhantomData<Type>);

/// Helper trait for `StorageInspect` to provide user-friendly API to retrieve storage as reference.
///
/// # Example
///
/// ```rust
/// use fuel_storage::{Mappable, Storage, StorageAsRef};
///
/// pub struct Contracts;
///
/// impl Mappable for Contracts {
///     type Key = [u8; 32];
///     type SetValue = [u8];
///     type GetValue = Vec<u8>;
/// }
///
/// pub struct Balances;
///
/// impl Mappable for Balances {
///     type Key = u128;
///     type SetValue = u64;
///     type GetValue = u64;
/// }
///
/// pub trait Logic: Storage<Contracts> + Storage<Balances> {
///     fn run(&self) {
///         // You can specify which `Storage` do you want to call with `storage::<Type>()`
///         let _ = self.storage::<Contracts>().get(&[0; 32]);
///         let _ = self.storage::<Balances>().get(&123);
///     }
/// }
/// ```
pub trait StorageAsRef<Error> {
    #[inline(always)]
    fn storage<Type>(&self) -> StorageRef<Self, Type>
    where
        Self: StorageInspect<Type, Error = Error>,
        Type: Mappable,
    {
        StorageRef(self, Default::default())
    }
}

/// The wrapper around the `Storage` that supports methods from `StorageInspect` and `StorageMutate`.
pub struct StorageMut<'a, T: 'a + ?Sized, Type: Mappable>(
    &'a mut T,
    core::marker::PhantomData<Type>,
);

/// Helper trait for `StorageMutate` to provide user-friendly API to retrieve storage as mutable reference.
///
/// # Example
///
/// ```rust
/// use fuel_storage::{Mappable, Storage, StorageAsMut};
///
/// pub struct Contracts;
///
/// impl Mappable for Contracts {
///     type Key = [u8; 32];
///     type SetValue = [u8];
///     type GetValue = Vec<u8>;
/// }
///
/// pub struct Balances;
///
/// impl Mappable for Balances {
///     type Key = u128;
///     type SetValue = u64;
///     type GetValue = u64;
/// }
///
/// pub trait Logic: Storage<Contracts> + Storage<Balances> {
///     fn run(&mut self) {
///         let mut self_ = self;
///         // You can specify which `Storage` do you want to call with `storage::<Type>()`
///         let _ = self_.storage::<Balances>().insert(&123, &321);
///         let _ = self_.storage::<Contracts>().get(&[0; 32]);
///     }
/// }
/// ```
pub trait StorageAsMut<Error> {
    #[inline(always)]
    fn storage<Type>(&mut self) -> StorageMut<Self, Type>
    where
        Self: Storage<Type, Error = Error>,
        Type: Mappable,
    {
        StorageMut(self, Default::default())
    }
}
