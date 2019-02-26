pub enum SessionManagerError {
    /// A poisoned mutex error; not much you can do for this except restart
    MutexSyncError,

    /// Conflict with an existing game name
    NameAlreadyInUse,

    /// No match for the master id that was requested
    NoMatchingMaster,

    /// No match for the client id that was requested
    NoMatchingClient,
}