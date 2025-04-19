# type: ignore
def match_datetime_component(
    temporal_function: TemporalFunction,
) -> tuple[DataType, u32 | None] | None:
    ({
        Millennium: (DataType.UInt32, None),
        Century: (DataType.Int32, None),
        Year: (DataType.Int32, None),
        IsoYear: (DataType.Int32, None),
        Quarter: (DataType.Int8, 4),
        Month: (DataType.Int8, 12),
        Week: (DataType.Int8, 53),
        WeekDay: (DataType.Int8, 7),
        Day: (DataType.Int8, 31),
        OrdinalDay: (DataType.Int16, 366),
        Hour: (DataType.Int8, 24),
        Minute: (DataType.Int8, 60),
        Second: (DataType.Int8, 60),
        Millisecond: (DataType.Int32, 1_000),
        Microsecond: (DataType.Int32, 1_000_000),
        Nanosecond: (DataType.Int32, 1_000_000_000),
    }).get(temporal_function)

