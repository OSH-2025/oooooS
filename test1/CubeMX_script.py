Import("env")

env.Append(LINKFLAGS=[
    "--specs=nosys.specs"
])