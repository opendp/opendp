#include <R.h>
#include <stdlib.h>
#include "ownership.h"

typedef struct Resource {
    void *ptr;
    void (*release)(void *);
    struct Resource *next;
} Resource;

typedef struct ResourceScope {
    Resource *resources;
    struct ResourceScope *previous;
} ResourceScope;

static ResourceScope *current_scope = NULL;

static void release_scope(void *data)
{
    ResourceScope *scope = (ResourceScope *)data;
    while (scope->resources)
    {
        Resource *resource = scope->resources;
        scope->resources = resource->next;
        resource->release(resource->ptr);
        free(resource);
    }
    current_scope = scope->previous;
}

SEXP opendp_with_scope(SEXP (*body)(void *), void *data)
{
    ResourceScope scope = {.resources = NULL, .previous = current_scope};
    current_scope = &scope;
    // Cleanup runs on normal returns and R longjmps, including errors during
    // argument conversion. R callbacks may enter another generated wrapper.
    return R_ExecWithCleanup(body, data, release_scope, &scope);
}

void opendp_register_resource(void *ptr, void (*release)(void *))
{
    if (!ptr)
        return;
    Resource *resource = (Resource *)malloc(sizeof(Resource));
    if (!current_scope || !resource)
    {
        free(resource);
        release(ptr);
        error("failed to register an OpenDP temporary resource");
    }
    resource->ptr = ptr;
    resource->release = release;
    resource->next = current_scope->resources;
    current_scope->resources = resource;
}

void opendp_forget_resource(void *ptr)
{
    if (!current_scope)
        error("OpenDP ownership transfer outside a resource scope");
    for (Resource **node = &current_scope->resources; *node; node = &(*node)->next)
    {
        if ((*node)->ptr == ptr)
        {
            Resource *resource = *node;
            *node = resource->next;
            free(resource);
            return;
        }
    }
    error("OpenDP ownership transfer of an unregistered resource");
}
