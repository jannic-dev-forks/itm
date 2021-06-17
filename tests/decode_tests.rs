use itm_decode::*;

#[test]
fn decode_sync_packet() {
    let mut trace_data: Vec<u8> = [0; 47 / 8].to_vec();
    trace_data.push(1 << 7);

    let mut decoder = Decoder::new();
    decoder.push(trace_data);
    assert_eq!(decoder.pull(), Ok(Some(TracePacket::Sync)));
}

#[test]
fn decode_overflow_packet() {
    let mut decoder = Decoder::new();
    decoder.push([0b0111_0000].to_vec());
    assert_eq!(decoder.pull(), Ok(Some(TracePacket::Overflow)));
}

#[test]
fn decode_local_timestamp_packets() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            // LTS1
            0b1100_0000,
            0b1100_1001,
            0b0000_0001,

            // LTS2
            0b0101_0000,
        ].to_vec());

    for packet in [
        TracePacket::LocalTimestamp1 {
            ts: 0b11001001,
            data_relation: TimestampDataRelation::Sync,
        },
        TracePacket::LocalTimestamp2 { ts: 0b101 },
    ]
    .iter()
    {
        assert_eq!(decoder.pull(), Ok(Some(packet.clone())));
    }
}

#[test]
fn decode_global_timestamp_packets() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            // GTS1
            0b1001_0100,
            0b1000_0000,
            0b1010_0000,
            0b1000_0100,
            0b0110_0000,

            // GTS2 (48-bit)
            0b1011_0100,
            0b1011_1101,
            0b1111_0100,
            0b1001_0001,
            0b0000_0001,

            // GTS2 (64-bit)
            0b1011_0100,
            0b1011_1101,
            0b1111_0100,
            0b1001_0001,
            0b1000_0001,
            0b1111_0100,
            0b0000_0111,
        ].to_vec());

    for packet in [
        TracePacket::GlobalTimestamp1 {
            ts: 0b00000_0000100_0100000_0000000,
            wrap: true,
            clkch: true,
        },
        TracePacket::GlobalTimestamp2 {
            ts: 0b1_0010001_1110100_0111101,
        },
        TracePacket::GlobalTimestamp2 {
            ts: 0b111_1110100_0000001_0010001_1110100_0111101,
        },
    ]
    .iter()
    {
        assert_eq!(decoder.pull(), Ok(Some(packet.clone())));
    }
}

#[test]
fn decode_extention_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b0111_1000,
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::Extension { page: 0b111 }))
    );
}

#[test]
fn decode_instrumentation_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b1000_1011,
            0b0000_0011,
            0b0000_1111,
            0b0011_1111,
            0b1111_1111,
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::Instrumentation {
            port: 0b1000_1,
            #[rustfmt::skip]
                payload: [
                    0b0000_0011,
                    0b0000_1111,
                    0b0011_1111,
                    0b1111_1111,
                ].to_vec(),
        }))
    );
}

#[test]
fn decode_eventcounterwrap_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b0000_0101,
            0b0010_1010,
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::EventCounterWrap {
            cyc: true,
            fold: false,
            lsu: true,
            sleep: false,
            exc: true,
            cpi: false,
        }))
    );
}

#[test]
fn decode_exceptiontrace_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b0000_1110,
            0b0010_0000,
            0b0011_0000
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::ExceptionTrace {
            exception: cortex_m::VectActive::Interrupt { irqn: 32 },
            action: ExceptionAction::Returned,
        }))
    );
}

#[test]
fn decode_pcsample_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            // PC sample (not sleeping)
            0b0001_0111,
            0b0000_0011,
            0b0000_1111,
            0b0011_1111,
            0b1111_1111,

            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,
        ].to_vec());

    for packet in [
        TracePacket::PCSample {
            pc: Some(0b11111111_00111111_00001111_00000011),
        },
        TracePacket::PCSample { pc: None },
    ]
    .iter()
    {
        assert_eq!(decoder.pull(), Ok(Some(packet.clone())));
    }
}

#[test]
fn decode_datatracepc_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b0111_0111,
            0b0000_0011,
            0b0000_1111,
            0b0011_1111,
            0b1111_1111,
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::DataTracePC {
            comparator: 0b11,
            pc: 0b11111111_00111111_00001111_00000011,
        }))
    );
}

#[test]
fn decode_datatraceaddress_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            0b0110_1110,
            0b0000_0011,
            0b0000_1111,
        ].to_vec());

    assert_eq!(
        decoder.pull(),
        Ok(Some(TracePacket::DataTraceAddress {
            comparator: 0b10,
            #[rustfmt::skip]
                data: [
                    0b0000_0011,
                    0b0000_1111,
                ].to_vec(),
        }))
    );
}

#[test]
fn decode_datatracevalue_packet() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            // four-byte (word) payload
            0b1010_1111,
            0b0000_0011,
            0b0000_1111,
            0b0011_1111,
            0b1111_1111,

            // two-byte (halfword) payload
            0b1010_1110,
            0b0000_0011,
            0b0000_1111,

            // one-byte (byte) payload
            0b1010_1101,
            0b0000_0011,
        ].to_vec());

    for packet in [
        TracePacket::DataTraceValue {
            comparator: 0b10,
            access_type: MemoryAccessType::Write,
            #[rustfmt::skip]
                value: [
                    0b0000_0011,
                    0b0000_1111,
                    0b0011_1111,
                    0b1111_1111,
                ].to_vec(),
        },
        TracePacket::DataTraceValue {
            comparator: 0b10,
            access_type: MemoryAccessType::Write,
            #[rustfmt::skip]
                value: [
                    0b0000_0011,
                    0b0000_1111,
                ].to_vec(),
        },
        TracePacket::DataTraceValue {
            comparator: 0b10,
            access_type: MemoryAccessType::Write,
            #[rustfmt::skip]
                value: [
                    0b0000_0011,
                ].to_vec(),
        },
    ]
    .iter()
    {
        assert_eq!(decoder.pull(), Ok(Some(packet.clone())));
    }
}

#[test]
fn pull_with_timestamp() {
    let mut decoder = Decoder::new();
    #[rustfmt::skip]
        decoder.push([
            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // GTS1
            0b1001_0100,
            0b1000_0000,
            0b1010_0000,
            0b1000_0100,
            0b0000_0000,

            // GTS2 (48-bit)
            0b1011_0100,
            0b1011_1101,
            0b1111_0100,
            0b1001_0001,
            0b0000_0001,

            // LTS1
            0b1100_0000,
            0b1100_1001,
            0b0000_0001,

            // Pull!

            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // LTS1
            0b1100_0000,
            0b1100_1001,
            0b0000_0001,

            // Pull!

            // Overflow
            0b0111_0000,

            // LTS1
            0b1100_0000,
            0b1100_1001,
            0b0000_0001,

            // Pull!

            // GTS1
            0b1001_0100,
            0b1000_0000,
            0b1010_0000,
            0b1000_0100,
            0b0000_0000,

            // GTS2 (48-bit)
            0b1011_0100,
            0b1011_1101,
            0b1111_0100,
            0b1001_0001,
            0b0000_0001,

            // LTS1
            0b1111_0000,
            0b1100_1001,
            0b0000_0001,

            // Pull!

            // Pull!
        ].to_vec());

    for set in [
        Ok(Some(TimestampedTracePackets {
            packets: [
                TracePacket::PCSample { pc: None },
                TracePacket::PCSample { pc: None },
                TracePacket::PCSample { pc: None },
            ]
            .into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: Some(0b1_1001001),
                data_relation: Some(TimestampDataRelation::Sync),
                diverged: false,
            },
        })),
        Ok(Some(TimestampedTracePackets {
            packets: [TracePacket::PCSample { pc: None }].into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: Some(0b1_1001001 * 2),
                data_relation: Some(TimestampDataRelation::Sync),
                diverged: false,
            },
        })),
        Ok(Some(TimestampedTracePackets {
            packets: [TracePacket::Overflow].into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: Some(0b1_1001001 * 3),
                data_relation: Some(TimestampDataRelation::Sync),
                diverged: true,
            },
        })),
        Ok(Some(TimestampedTracePackets {
            packets: [].into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: Some(0b1_1001001),
                data_relation: Some(TimestampDataRelation::UnknownAssocEventDelay),
                diverged: false,
            },
        })),
        Ok(None),
    ]
    .iter()
    {
        assert_eq!(decoder.pull_with_timestamp(), *set);
    }
}

#[test]
fn pull_with_timestamp_gts_only() {
    let mut decoder = Decoder::new();
    decoder.only_global_timestamps(true);
    #[rustfmt::skip]
        decoder.push([
            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // Pull!

            // GTS1
            0b1001_0100,
            0b1000_0000,
            0b1010_0000,
            0b1000_0100,
            0b0000_0000,

            // GTS2 (48-bit)
            0b1011_0100,
            0b1011_1101,
            0b1111_0100,
            0b1001_0001,
            0b0000_0001,

            // PC sample (sleeping)
            0b0001_0101,
            0b0000_0000,

            // Pull!

            // LTS1
            0b1100_0000,
            0b1100_1001,
            0b0000_0001,

            // Pull!

            // Pull!
        ].to_vec());

    for set in [
        Ok(Some(TimestampedTracePackets {
            packets: [TracePacket::PCSample { pc: None }].into(),
            timestamp: Timestamp {
                base: None,
                delta: None,
                data_relation: None,
                diverged: false,
            },
        })),
        Ok(Some(TimestampedTracePackets {
            packets: [TracePacket::PCSample { pc: None }].into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: None,
                data_relation: None,
                diverged: false,
            },
        })),
        Ok(Some(TimestampedTracePackets {
            packets: [TracePacket::LocalTimestamp1 {
                ts: 201,
                data_relation: TimestampDataRelation::Sync,
            }]
            .into(),
            timestamp: Timestamp {
                base: Some((0b1_0010001_1110100_0111101 << 26) | (0b0_0000100_0100000_0000000)),
                delta: None,
                data_relation: None,
                diverged: false,
            },
        })),
        Ok(None),
    ]
    .iter()
    {
        assert_eq!(decoder.pull_with_timestamp(), *set);
    }
}
