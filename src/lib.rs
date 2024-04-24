use std::{error::Error, future::Future, marker::PhantomData};

use serde::{Deserialize, Serialize};

// Analagous to a configured terraform provider.
// Any common configuration for managing resources associated with this provider.
// It is recommended to use the `secrecy` library or some alternative to protect user secrets.
pub trait Provider {}

// A specific type of provider resource. For example, a VPC.
#[trait_variant::make(ResourceType: Send + Sync)]
pub trait LocalResourceType<'a> {
    type ResourceProvider: Provider;

    // All the information needed to create an instance of this resource
    type ResourceSpec;

    // Information about this resource's state obtained from the provider
    type ResourceState: Serialize + Deserialize<'a>;

    // Enough info to locate the resource so it can be read, updated, or deleted
    type ResourceIdentifier;

    type CreateError: Error;
    type GetError: Error;
    type UpdateError: Error;
    type DeleteError: Error;

    // Create a new 
    // Not all providers will need a reqwest client, but it's common enough that
    // it's included here. Might result in some extra boilerplate for uncommon use cases.
    // If you need something else, consider putting it in the ResourceProvider
    async fn create(
        client: reqwest::Client,
        provider: Self::ResourceProvider
    ) -> Result<(Self::ResourceIdentifier, Self::ResourceState), Self::CreateError>;

    // Fetch the current state of the resource from the provider.
    // Not all providers will need a reqwest client, but it's common enough that
    // it's included here. Might result in some extra boilerplate for uncommon use cases.
    // If you need something else, consider putting it in the ResourceProvider
    async fn get(
        id: Self::ResourceIdentifier,
        client: reqwest::Client,
        provider: Self::ResourceProvider
    ) -> Result<Self::ResourceState, Self::GetError>;

    // Update the indicated resource, then return the resulting state if successful.
    // Not all providers will need a reqwest client, but it's common enough that
    // it's included here. Might result in some extra boilerplate for uncommon use cases.
    // If you need something else, consider putting it in the ResourceProvider
    async fn update(
        id: Self::ResourceIdentifier,
        spec: Self::ResourceSpec,
        client: reqwest::Client,
        provider: Self::ResourceProvider
    ) -> Result<Self::ResourceState, Self::UpdateError>;

    // Delete the identified resource.
    // Not all providers will need a reqwest client, but it's common enough that
    // it's included here. Might result in some extra boilerplate for uncommon use cases.
    // If you need something else, consider putting it in the ResourceProvider
    async fn delete(
        id: Self::ResourceIdentifier,
        client: reqwest::Client,
        provider: Self::ResourceProvider
    ) -> Result<(), Self::DeleteError>;
}

pub enum Resource<'a, R: ResourceType<'a>, DL: DependencyList, RB: ResourceBuilder<'a, R, DL>> {
    AwaitingDeps(DL),
    Building(R::ResourceSpec, Box<dyn Future<Output = R::ResourceState>>),
    Done(R::ResourceSpec, R::ResourceState),
    _NotUsed(PhantomData<RB>),
}

pub struct Dependency {

}

trait DependencyTupleTrait {}

impl DependencyTupleTrait for () {}

impl<Tail: DependencyTupleTrait > DependencyTupleTrait for (Dependency, Tail) {}

// A collection of dependencies. Under the hood, we use tuple structs for type checking and
// iteration, but we don't want users to have to work with tuple structs when building resources.
// TODO: write a derive macro for this.
pub trait DependencyList {
    #[allow(private_bounds)]
    type DependencyTuple: DependencyTupleTrait;

    fn tupleify(self) -> Self::DependencyTuple;
    fn detupleify(self) -> Self::DependencyTuple;
}

// A struct that creates a resource spec once its dependencies have been created.
// This is intended to be implemented manually by users, though providers may wish to
// provide implementations for common use cases of their products.
#[trait_variant::make(ResourceBuilder: Send + Sync)]
pub trait LocalResourceBuilder<'a, R: ResourceType<'a>, DL: DependencyList> {
    
    // Build a resource spec once the dependencies have been created.
    async fn build_spec(dependencies: DL) -> R::ResourceSpec;
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn it_works() {
    // }
}
