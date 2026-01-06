plugins {
    java
    application
}

group = "com.npcsociety"
version = "1.0.0-SNAPSHOT"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

repositories {
    mavenCentral()
}

val grpcVersion = "1.72.0"
val protobufVersion = "4.33.2"

dependencies {
    // gRPC
    implementation("io.grpc:grpc-netty-shaded:$grpcVersion")
    implementation("io.grpc:grpc-protobuf:$grpcVersion")
    implementation("io.grpc:grpc-stub:$grpcVersion")
    
    // Protobuf
    implementation("com.google.protobuf:protobuf-java:$protobufVersion")
    
    // Annotations for generated code
    compileOnly("org.apache.tomcat:annotations-api:6.0.53")
}

// Include generated sources
sourceSets {
    main {
        java {
            srcDir("../../gen/java")
        }
    }
}

application {
    mainClass.set("com.npcsociety.examples.ExampleClient")
}
