# type: ignore
def match_datetime_component(
    temporal_function: TemporalFunction,
) -> DataType | None:
    return ({
        Millennium: DataType.UInt32,
        Century: DataType.Int32,
        Year: DataType.Int32,
        IsoYear: DataType.Int32,
        Quarter: DataType.Int8,
        Month: DataType.Int8,
        Week: DataType.Int8,
        WeekDay: DataType.Int8,
        Day: DataType.Int8,
        OrdinalDay: DataType.Int16,
        Hour: DataType.Int8,
        Minute: DataType.Int8,
        Second: DataType.Int8,
        Millisecond: DataType.Int32,
        Microsecond: DataType.Int32,
        Nanosecond: DataType.Int32,
    }).get(temporal_function)

