//! Identity tests.

use multi_asi_immune::identity::keypair::AsiIdentity;

#[test]
fn test_identity_generation() {
    let id1 = AsiIdentity::generate();
    let id2 = AsiIdentity::generate();
    
    assert_ne!(id1.id, id2.id);
}

#[test]
fn test_sign_and_verify() {
    let identity = AsiIdentity::generate();
    let message = b"test message for signing";
    
    let signature = identity.sign(message);
    let public = identity.public_identity();
    
    assert!(public.verify(message, &signature));
}

#[test]
fn test_verification_fails_wrong_identity() {
    let id1 = AsiIdentity::generate();
    let id2 = AsiIdentity::generate();
    
    let message = b"test";
    let sig = id1.sign(message);
    
    assert!(!id2.public_identity().verify(message, &sig));
}

#[test]
fn test_verification_fails_wrong_message() {
    let identity = AsiIdentity::generate();
    
    let sig = identity.sign(b"original");
    let public = identity.public_identity();
    
    assert!(!public.verify(b"modified", &sig));
}

#[test]
fn test_deterministic_id_from_secret() {
    let secret = [42u8; 32];
    
    let id1 = AsiIdentity::from_bytes(&secret);
    let id2 = AsiIdentity::from_bytes(&secret);
    
    assert_eq!(id1.id, id2.id);
}

#[test]
fn test_id_display() {
    let identity = AsiIdentity::generate();
    let display = format!("{}", identity.id);
    
    assert_eq!(display.len(), 8); // Short hex
}
