# Enclave Support

## What is enclave support?
[Enclave](https://en.wikipedia.org/wiki/Network_enclave): *The purpose of a network enclave is to limit internal access to a portion of a network. 

A major difference between a DMZ or demilitarized zone and a network enclave is a DMZ allows inbound and outbound traffic access, where firewall boundaries are traversed. 
In an enclave, firewall boundaries are not traversed.*

oc-mirror already focuses on mirroring content to disconnected environments for installing and upgrading OCP clusters.

This specific feature addresses use cases where mirroring is needed for several enclaves (disconnected environments), that are secured behind at least one intermediate disconnected network. 

In this context, enclave users are interested in:
* being able to **mirror content for several enclaves**, and centralizing it in a single internal registry. Some customers are interested in running security checks on the mirrored content, 
   vetting it before allowing mirroring to downstream enclaves.
* being able to mirror contents **directly from the internal centralized registry** to enclaves without having to restart the mirroring from internet for each enclave
* **keeping the volume** of data transfered from one network stage to the other **to a strict minimum**, avoiding to transfer a blob or an image more than one time from one stage to another.

## When can I use the enclave support feature?
:warning: **The Enclave support feature is an MVP in Developer Preview and should not be used in production.**

In the OpenShift 4.16 release, the enclave workflow is in Tech Preview. 

After GA, this new mirroring technique is intended to replace the existing oc-mirror. 

To enable the enclave workflow, add `--v2` to the oc-mirror arguments passed at the command line.

Example:
```bash=
oc-mirror --v2 --help
```

