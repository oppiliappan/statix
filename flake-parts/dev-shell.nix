{
  partitionedAttrs.devShells = "dev";
  partitions.dev.module = devPartition: {
    imports = [
      devPartition.inputs.make-shell.flakeModules.default
    ];
  };
}
